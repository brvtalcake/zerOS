//! # Memory management in zerOS
//!
//! Generally, here is how an allocation from a collection in the `alloc` crate
//! would be satisfied:
//! - First, ask the `GlobalAllocator`*
//! - Then the `GlobalAllocator` asks for some memory, if his managed memory
//!   isn't enough
//! - Then some suballocator traverses an `RBTree` of different regions
//!   (previously feeded to the global `PageAllocator`, which himself could
//!   maybe be able to ask the virtual memory manager to get some more pages,
//!   and potentially to give him back under memory pressure)
//! - Then one of the traversed regions then allocates some pages
//!
//! * (the name of the `GlobalAllocator` is yet to be defined)
//!
//! This architecture enables us to use different allocation strategies (and
//! different allocators) based on the type/size of data that needs to be
//! stored. It also makes it possible to plug random memory regions to the
//! `PageAllocator`

use alloc::alloc::{AllocError, Allocator, Layout};
use core::{
	cell::RefCell,
	cmp,
	num::NonZero,
	ptr::{self, copy_nonoverlapping, NonNull}
};

use intrusive_collections::{
	KeyAdapter,
	LinkedList,
	LinkedListLink,
	RBTree,
	RBTreeLink,
	UnsafeRef,
	intrusive_adapter
};
use lazy_static::lazy_static;
use strength_reduce::StrengthReducedUsize;

use crate::{arch::target::PAGE_SIZE, utils::with_lifetime_mut};

#[derive(Clone, Copy, PartialEq, Eq)]
struct MemorySubRegionHeaderInfo
{
	page_count: NonZero<usize>,
	free:       bool
}

impl MemorySubRegionHeaderInfo {}

impl PartialOrd<Self> for MemorySubRegionHeaderInfo
{
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering>
	{
		match (self.free, other.free)
		{
			// just separate them in the tree
			// saying that those that aren't free are *LESS* than those that are free, means that
			// advancing though rbtree's `lower_bound()` requests will only give us the free chunks
			(true, false) => Some(cmp::Ordering::Greater),
			(false, true) => Some(cmp::Ordering::Less),
			_ => self.page_count.partial_cmp(&other.page_count)
		}
	}
}

impl Ord for MemorySubRegionHeaderInfo
{
	fn cmp(&self, other: &Self) -> cmp::Ordering
	{
		self.partial_cmp(other).unwrap()
	}
}

struct MemorySubRegionHeader
{
	list_link:   LinkedListLink,
	rbtree_link: RBTreeLink,
	// TODO: once we can prove that we can't panic on double `borrow_mut()`, switch the following
	// to `Cell` instead
	info:        RefCell<MemorySubRegionHeaderInfo>
}

impl MemorySubRegionHeader
{
	fn as_ptr(&self) -> *const Self
	{
		&raw const *self
	}

	fn as_mut_ptr(&mut self) -> *mut Self
	{
		&raw mut *self
	}

	fn chunk_limits(&self) -> (usize, usize)
	{
		(
			self.as_ptr() as usize,
			self.as_ptr() as usize
				+ unsafe {
					self.info
						.borrow_mut()
						.page_count
						.unchecked_mul(NonZero::new_unchecked(PAGE_SIZE))
						.get()
				}
		)
	}

	fn user_chunk_limits(&self) -> (usize, usize)
	{
		let (start, end) = self.chunk_limits();
		(start + size_of::<Self>(), end)
	}
}

intrusive_adapter! {
	MemorySubRegionHeaderRBTreeAdapter
		= UnsafeRef<MemorySubRegionHeader>:
			MemorySubRegionHeader {
				rbtree_link: RBTreeLink
			}
}

intrusive_adapter! {
	MemorySubRegionHeaderLinkedListAdapter
		= UnsafeRef<MemorySubRegionHeader>:
			MemorySubRegionHeader {
				list_link: LinkedListLink
			}
}

impl<'a> KeyAdapter<'a> for MemorySubRegionHeaderRBTreeAdapter
{
	type Key = MemorySubRegionHeaderInfo;

	fn get_key(
		&self,
		value: &'a <Self::PointerOps as intrusive_collections::PointerOps>::Value
	) -> Self::Key
	{
		*value.info.borrow()
	}
}

struct RegionAllocator
{
	rbtree: RefCell<RBTree<MemorySubRegionHeaderRBTreeAdapter>>,
	list:   RefCell<LinkedList<MemorySubRegionHeaderLinkedListAdapter>>,
	region: RefCell<&'static mut [u8]>,
	size:   NonZero<usize>
}

// TODO: this might not be worth the trouble since the compiler should be clever
// enough to do it himself
lazy_static! {
	static ref PAGE_SIZE_FAST: StrengthReducedUsize = StrengthReducedUsize::new(PAGE_SIZE);
}

impl RegionAllocator
{
	fn initial_sanity_checks(region: &'_ [u8]) -> bool
	{
		let start = region.as_ptr() as usize;
		let len = region.len();
		(start % *PAGE_SIZE_FAST == 0) && (len % *PAGE_SIZE_FAST == 0) && (len > 2 * PAGE_SIZE)
	}

	// fn split_chunk(&mut self, orig: *mut MemorySubRegionHeader, split_at: usize)

	/// # SAFETY
	/// The caller must ensure that the managed region remains valid, live, and
	/// untouched while being used by the allocator
	pub unsafe fn new<'a>(region: &'a mut [u8], region_header: bool) -> Option<Self>
	{
		if let Some(this) = Self::initial_sanity_checks(region).then_some(Self {
			rbtree: RefCell::new(RBTree::new(MemorySubRegionHeaderRBTreeAdapter::default())),
			list:   RefCell::new(LinkedList::new(
				MemorySubRegionHeaderLinkedListAdapter::default()
			)),
			region: RefCell::new(unsafe { with_lifetime_mut(region) }),
			size:   region.len().try_into().ok()?
		})
		{
			if region_header
			{
				// TODO: reserve the first page to hold a `MemoryRegionHeader`, and return it
				todo!("this is yet to be implemented !")
			}
			else
			{
				let hdr = MemorySubRegionHeader {
					list_link:   Default::default(),
					rbtree_link: Default::default(),
					info:        RefCell::new(MemorySubRegionHeaderInfo {
						page_count: (region.len() / *PAGE_SIZE_FAST).try_into().ok()?,
						free:       true
					})
				};
				let val = unsafe {
					copy_nonoverlapping(
						&raw const hdr,
						this.region.borrow_mut().as_mut_ptr().cast(),
						1
					);
					this.region
						.borrow()
						.as_ptr()
						.cast::<MemorySubRegionHeader>()
				};
				this.list
					.borrow_mut()
					.push_front(unsafe { UnsafeRef::from_raw(val) });
				this.rbtree
					.borrow_mut()
					.insert(unsafe { UnsafeRef::from_raw(val) });
				Some(this)
			}
		}
		else
		{
			None
		}
	}

	pub fn allocate_best_fit(&self, layout: &Layout) -> Result<NonNull<[u8]>, AllocError>
	{
		let layout_size = layout.size();
		let layout_align = layout.align();
		let required_minimum_size = layout_size + size_of::<MemorySubRegionHeader>();

		#[rustfmt::skip]
		// region must contain space for `layout_size` + the size of the memory header
		let region_bound = unsafe {
			MemorySubRegionHeaderInfo {
				page_count: <NonZero<usize>>::new_unchecked(
					(required_minimum_size / *PAGE_SIZE_FAST)
						+ (
							if (required_minimum_size % *PAGE_SIZE_FAST) == 0
							{ 0 }
							else
							{ 1 }
						)
				),
				free: true
			}
		};
		let bound = intrusive_collections::Bound::Included(&region_bound);
		let mut rbtree = self.rbtree.borrow_mut();
		let mut list = self.list.borrow_mut();
		let mut rbtree_cursor = rbtree.lower_bound_mut(bound);
		loop
		{
			if rbtree_cursor.is_null()
			{
				break;
			}
			let currnode = unsafe { rbtree_cursor.get().unwrap_unchecked() };
			let (currstart, currend) = currnode.user_chunk_limits();
			let aligned = (currstart - 1 + layout_align) & !(layout_align - 1);
			if aligned + layout_size <= currend
			{
				if (aligned + layout_size - currend) / *PAGE_SIZE_FAST > 1
				{
					// split the chunk
				}
				else
				{
					// mark as used an do not split the chunk
					let removed = UnsafeRef::<MemorySubRegionHeader>::into_raw(
						rbtree_cursor.remove().unwrap()
					);
					unsafe {
						(*removed).info.borrow_mut().free = false;
						rbtree.insert(UnsafeRef::from_raw(removed.cast_const()));
						let ret: *mut [u8] = ptr::slice_from_raw_parts_mut(aligned as *mut u8, layout_size);
						return Ok(NonNull::new_unchecked(ret));
					}
				}
			}
			rbtree_cursor.move_next()
		}
		Err(AllocError)
	}

	pub fn allocate_first_fit(&self, layout: &Layout) -> Result<NonNull<[u8]>, AllocError>
	{
		Err(AllocError)
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct MemoryRegionHeaderInfo
{
	page_count: NonZero<usize>,
	free:       bool
}

impl MemoryRegionHeaderInfo {}

struct MemoryRegionHeader
{
	list_link:   LinkedListLink,
	rbtree_link: RBTreeLink,
	// TODO: once we can prove that we can't panic on double `borrow_mut()`, switch the following
	// to `Cell` instead
	info:        RefCell<MemoryRegionHeaderInfo>
}
