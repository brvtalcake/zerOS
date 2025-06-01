use alloc::alloc::{AllocError, Allocator, Layout};
use core::{
	alloc::GlobalAlloc,
	cell::UnsafeCell,
	ptr::{self, NonNull}
};

use super::allocators::{AllocationStrategy, RegionAllocator, RegionAllocatorReclaimHook};
use crate::{arch::target, kernel::sync::BasicMutex};

struct RegionAllocatorListLink
{
	prev:  Option<NonNull<RegionAllocatorListLink>>,
	next:  Option<NonNull<RegionAllocatorListLink>>,
	value: Option<NonNull<RegionAllocator>>
}

struct RegionAllocatorList
{
	head:            Option<NonNull<RegionAllocatorListLink>>,
	tail:            Option<NonNull<RegionAllocatorListLink>>,
	last_successful: Option<NonNull<RegionAllocatorListLink>>
}

impl RegionAllocatorList
{
	fn add_to_end(
		&mut self,
		allocator: NonNull<RegionAllocator>,
		link: NonNull<RegionAllocatorListLink>
	)
	{
		let curr_tail = self.tail.unwrap();
		let curr_tail_link = unsafe { curr_tail.read() };
		debug_assert!(curr_tail_link.next.is_none());

		unsafe {
			link.write(RegionAllocatorListLink {
				prev:  Some(curr_tail),
				next:  None,
				value: Some(allocator)
			});
			curr_tail.write(RegionAllocatorListLink {
				next: Some(link),
				..curr_tail_link
			});
		}

		self.tail = Some(link);
	}

	fn add_region(
		&mut self,
		region: &'static mut [u8],
		static_memory: bool,
		reclaim_hook: RegionAllocatorReclaimHook,
		reclaimable: bool,
		default_strategy: AllocationStrategy
	) -> Option<()>
	{
		debug_assert!(
			self.head.is_some(),
			"the `head` field of `RegionAllocatorList` should always be `Some(non-null-ptr)` \
			 after it has been initialized ! (i.e. it should always contain at least one \
			 `RegionAllocator`)"
		);
		debug_assert!(
			self.tail.is_some(),
			"the `tail` field of `RegionAllocatorList` should always be `Some(non-null-ptr)` \
			 after it has been initialized ! (i.e. it should always contain at least one \
			 `RegionAllocator`)"
		);
		debug_assert!(
			self.last_successful.is_some(),
			"the `last_successful` field of `RegionAllocatorList` should always be \
			 `Some(non-null-ptr)` after it has been initialized ! (i.e. it should always contain \
			 at least one `RegionAllocator`)"
		);

		let tmp_link = RegionAllocatorListLink {
			prev:  None,
			next:  None,
			value: None
		};
		let (new_region, new_link) = unsafe {
			RegionAllocator::new_extended(
				region,
				static_memory,
				reclaim_hook,
				reclaimable,
				default_strategy,
				tmp_link
			)
		}?;
		self.add_to_end(new_region, new_link);
		Some(())
	}
}

impl IntoIterator for RegionAllocatorList
{
	type IntoIter = RegionAllocatorIter;
	type Item = <RegionAllocatorIter as Iterator>::Item;

	fn into_iter(self) -> Self::IntoIter
	{
		<Self::IntoIter as From<RegionAllocatorList>>::from(self)
	}
}

impl IntoIterator for &RegionAllocatorList
{
	type IntoIter = RegionAllocatorIter;
	type Item = <RegionAllocatorIter as Iterator>::Item;

	fn into_iter(self) -> Self::IntoIter
	{
		<Self::IntoIter as From<&RegionAllocatorList>>::from(self)
	}
}

impl From<&RegionAllocatorList> for RegionAllocatorIter
{
	fn from(value: &RegionAllocatorList) -> Self
	{
		Self {
			list_head: value.head.unwrap(),
			first:     value.last_successful.unwrap(),
			current:   value.last_successful
		}
	}
}

impl From<RegionAllocatorList> for RegionAllocatorIter
{
	fn from(value: RegionAllocatorList) -> Self
	{
		Self {
			list_head: value.head.unwrap(),
			first:     value.last_successful.unwrap(),
			current:   value.last_successful
		}
	}
}

struct RegionAllocatorIter
{
	list_head: NonNull<RegionAllocatorListLink>,
	first:     NonNull<RegionAllocatorListLink>,
	current:   Option<NonNull<RegionAllocatorListLink>>
}

impl RegionAllocatorIter
{
	/// # Safety
	/// ## Required pre-conditions
	/// `self.current.is_some()` is true
	unsafe fn update_iter(&mut self) -> Option<<Self as Iterator>::Item>
	{
		let link = unsafe { self.current.unwrap().read() };
		let allocator = link.value.unwrap();
		let next = match link.next
		{
			Some(value) => value,
			_ => self.list_head
		};
		self.current = if next == self.first { None } else { Some(next) };
		Some(allocator)
	}
}

impl Iterator for RegionAllocatorIter
{
	type Item = NonNull<RegionAllocator>;

	fn next(&mut self) -> Option<Self::Item>
	{
		if self.current.is_some()
		{
			unsafe { self.update_iter() }
		}
		else
		{
			None
		}
	}
}

pub struct KernelAllocator
{
	regions: BasicMutex<RegionAllocatorList>
}

unsafe impl Sync for KernelAllocator {}
unsafe impl Send for KernelAllocator {}

#[overloadf::overload]
impl KernelAllocator
{
	pub const fn const_new() -> Self
	{
		Self {
			regions: BasicMutex::new(RegionAllocatorList {
				head:            None,
				tail:            None,
				last_successful: None
			})
		}
	}

	pub fn new(region: &'static mut [u8], static_memory: bool) -> Option<Self>
	{
		unsafe { Self::new(region, static_memory, AllocationStrategy::BestFit) }
	}

	pub fn new(
		region: &'static mut [u8],
		static_memory: bool,
		default_strategy: AllocationStrategy
	) -> Option<Self>
	{
		unsafe { Self::new(region, static_memory, None, false, default_strategy) }
	}

	pub fn new(
		region: &'static mut [u8],
		static_memory: bool,
		reclaim_hook: RegionAllocatorReclaimHook,
		reclaimable: bool,
		default_strategy: AllocationStrategy
	) -> Option<Self>
	{
		let (new_region, new_link) = unsafe {
			RegionAllocator::new_extended(
				region,
				static_memory,
				reclaim_hook,
				reclaimable,
				default_strategy,
				RegionAllocatorListLink {
					prev:  None,
					next:  None,
					value: None
				}
			)
		}?;
		unsafe {
			new_link.write(RegionAllocatorListLink {
				prev:  None,
				next:  None,
				value: Some(new_region)
			});
		}
		Some(Self {
			regions: BasicMutex::new(RegionAllocatorList {
				head:            Some(new_link),
				tail:            Some(new_link),
				last_successful: Some(new_link)
			})
		})
	}

	pub fn add_region_basic(&self, region: &'static mut [u8], static_memory: bool)
	-> Option<&Self>
	{
		self.add_region(region, static_memory, AllocationStrategy::BestFit)
	}

	pub fn add_region(
		&self,
		region: &'static mut [u8],
		static_memory: bool,
		default_strategy: AllocationStrategy
	) -> Option<&Self>
	{
		self.add_region_extended(region, static_memory, None, false, default_strategy)
	}

	pub fn add_region_extended(
		&self,
		region: &'static mut [u8],
		static_memory: bool,
		reclaim_hook: RegionAllocatorReclaimHook,
		reclaimable: bool,
		default_strategy: AllocationStrategy
	) -> Option<&Self>
	{
		self.regions.lock().add_region(
			region,
			static_memory,
			reclaim_hook,
			reclaimable,
			default_strategy
		)?;
		Some(self)
	}
}

unsafe impl Allocator for KernelAllocator
{
	fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError>
	{
		let guard = self.regions.lock();
		let list: &RegionAllocatorList = &guard;
		for allocator in list.into_iter()
		{
			if let res @ Ok(_) = unsafe { allocator.as_ref().allocate(layout) }
			{
				return res;
			}
		}
		Err(AllocError)
	}

	unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout)
	{
		let guard = self.regions.lock();
		let list: &RegionAllocatorList = &guard;
		for allocator in list.into_iter()
		{
			unsafe {
				if allocator.as_ref().contains(ptr)
				{
					allocator.as_ref().deallocate(ptr, layout);
					return;
				}
			}
		}
	}

	fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError>
	{
		let guard = self.regions.lock();
		let list: &RegionAllocatorList = &guard;
		for allocator in list.into_iter()
		{
			if let res @ Ok(_) = unsafe { allocator.as_ref().allocate_zeroed(layout) }
			{
				return res;
			}
		}
		Err(AllocError)
	}

	unsafe fn shrink(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout
	) -> Result<NonNull<[u8]>, AllocError>
	{
		let guard = self.regions.lock();
		let list: &RegionAllocatorList = &guard;
		for allocator in list.into_iter()
		{
			unsafe {
				if allocator.as_ref().contains(ptr)
				{
					return allocator.as_ref().shrink(ptr, old_layout, new_layout);
				}
			}
		}
		Err(AllocError)
	}

	unsafe fn grow(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout
	) -> Result<NonNull<[u8]>, AllocError>
	{
		let guard = self.regions.lock();
		let list: &RegionAllocatorList = &guard;
		for allocator in list.into_iter()
		{
			unsafe {
				if allocator.as_ref().contains(ptr)
				{
					return allocator.as_ref().grow(ptr, old_layout, new_layout);
				}
			}
		}
		Err(AllocError)
	}

	unsafe fn grow_zeroed(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout
	) -> Result<NonNull<[u8]>, AllocError>
	{
		let guard = self.regions.lock();
		let list: &RegionAllocatorList = &guard;
		for allocator in list.into_iter()
		{
			unsafe {
				if allocator.as_ref().contains(ptr)
				{
					return allocator.as_ref().grow_zeroed(ptr, old_layout, new_layout);
				}
			}
		}
		Err(AllocError)
	}
}

unsafe impl GlobalAlloc for KernelAllocator
{
	unsafe fn alloc(&self, layout: Layout) -> *mut u8
	{
		self.allocate(layout)
			.map_or(ptr::null_mut(), |ptr| ptr.as_mut_ptr())
	}

	unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8
	{
		self.allocate_zeroed(layout)
			.map_or(ptr::null_mut(), |ptr| ptr.as_mut_ptr())
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout)
	{
		if !ptr.is_null()
		{
			unsafe {
				self.deallocate(NonNull::new_unchecked(ptr), layout);
			}
		}
	}

	unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8
	{
		if ptr.is_null()
		{
			ptr::null_mut()
		}
		else if new_size < layout.size()
		{
			unsafe {
				Layout::from_size_align(new_size, layout.align()).map_or(
					ptr::null_mut(),
					|new_layout| {
						self.shrink(NonNull::new_unchecked(ptr), layout, new_layout)
							.map_or(ptr::null_mut(), |ptr| ptr.as_mut_ptr())
					}
				)
			}
		}
		else if new_size == layout.size()
		{
			ptr
		}
		else if new_size > layout.size()
		{
			unsafe {
				Layout::from_size_align(new_size, layout.align()).map_or(
					ptr::null_mut(),
					|new_layout| {
						self.grow(NonNull::new_unchecked(ptr), layout, new_layout)
							.map_or(ptr::null_mut(), |ptr| ptr.as_mut_ptr())
					}
				)
			}
		}
		else
		{
			unsafe { core::hint::unreachable_unchecked() }
		}
	}
}

#[repr(align(4096))]
struct StaticRegion<const SIZE: usize>
{
	buffer: UnsafeCell<[u8; SIZE]>
}

impl<const SIZE: usize> StaticRegion<SIZE>
{
	const fn new() -> Self
	{
		Self {
			buffer: UnsafeCell::new([0; SIZE])
		}
	}

	const unsafe fn get_mut_buffer(&'static self) -> &'static mut [u8]
	{
		unsafe { self.buffer.as_mut_unchecked() }
	}
}

unsafe impl<const SIZE: usize> Sync for StaticRegion<SIZE> {}
unsafe impl<const SIZE: usize> Send for StaticRegion<SIZE> {}

static ZEROS_GLOBAL_ALLOCATOR_BASE_REGION: StaticRegion<{ target::PAGE_SIZE * 1028 * 2 }> =
	StaticRegion::new();

#[global_allocator]
pub static ZEROS_GLOBAL_ALLOCATOR: KernelAllocator = KernelAllocator::const_new();

ctor! {
	@priority(0);
	@name(zerOS_initialize_global_allocator);

	crate::arch::target::cpu::irq::disable();
	let new = KernelAllocator::new(
		unsafe { ZEROS_GLOBAL_ALLOCATOR_BASE_REGION.get_mut_buffer() },
		true
	).unwrap_or_else(
		|| {
			crate::arch::target::cpu::irq::enable();
			crate::arch::target::cpu::misc::hcf()
		}
	);
	unsafe {
		let global_allocator_ptr: *mut KernelAllocator = (&raw const ZEROS_GLOBAL_ALLOCATOR).cast_mut();
		ptr::write(global_allocator_ptr, new);
	}
	crate::arch::target::cpu::irq::enable();
}
