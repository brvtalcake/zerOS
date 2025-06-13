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
	ffi::c_void,
	hint::unlikely,
	ptr::{self, NonNull}
};

mod bindings;

pub(super) use bindings::region::zerOS_region_allocator_max_size_for;
pub use bindings::region::zerOS_region_reclaim_hook_t as RegionAllocatorReclaimHook;

#[derive(Default)]
pub enum AllocationStrategy
{
	#[default]
	Default,
	FirstFit,
	BestFit
}

impl AllocationStrategy
{
	fn to_binding_strategy(&self) -> bindings::region::zerOS_allocation_strategy
	{
		match self
		{
			Self::Default => bindings::region::zerOS_allocation_strategy::zerOS_ALLOC_STRAT_DEFAULT,
			Self::FirstFit =>
			{
				bindings::region::zerOS_allocation_strategy::zerOS_ALLOC_STRAT_FIRST_FIT
			},
			Self::BestFit => bindings::region::zerOS_allocation_strategy::zerOS_ALLOC_STRAT_BEST_FIT
		}
	}
}

#[repr(C)]
pub struct RegionAllocator
{
	pub(super) inner:
		*mut bindings::region::zerOS_region_allocator,
	self_storage: NonNull<Self>,
	after_self: NonNull<u8>,
	after_size: usize
}

impl Default for RegionAllocator
{
	fn default() -> Self
	{
		Self {
			inner:        ptr::null_mut(),
			self_storage: NonNull::dangling(),
			after_self:   NonNull::dangling(),
			after_size:   usize::MAX
		}
	}
}

impl RegionAllocator
{
	pub unsafe fn new(
		region: &'static mut [u8],
		static_memory: bool,
		reclaim_hook: RegionAllocatorReclaimHook,
		reclaimable: bool,
		default_strategy: AllocationStrategy
	) -> Option<NonNull<Self>>
	{
		use bindings::region::zerOS_region_allocator_create;
		let maybe_inner = unsafe {
			zerOS_region_allocator_create(
				region.as_mut_ptr(),
				region.len(),
				static_memory,
				reclaimable,
				default_strategy.to_binding_strategy(),
				reclaim_hook
			)
		};
		if unlikely(maybe_inner.is_null())
		{
			return None;
		}
		unsafe {
			let mut default = Self {
				inner: maybe_inner,
				..Default::default()
			};
			let (pointers, remaining) = default.get_self_ptr();
			if let Some((self_ptr, after_self)) = pointers
			{
				default.self_storage = self_ptr;
				default.after_self = after_self;
				default.after_size = remaining;
				let storage = default.write_self_to_additional_storage();
				Some(storage)
			}
			else
			{
				None
			}
		}
	}

	pub fn contains(&self, ptr: NonNull<u8>) -> bool
	{
		use bindings::region::zerOS_region_allocator_contains;
		unsafe { zerOS_region_allocator_contains(self.inner, ptr.as_ptr().cast()) }
	}

	pub unsafe fn new_extended<T>(
		region: &'static mut [u8],
		static_memory: bool,
		reclaim_hook: RegionAllocatorReclaimHook,
		reclaimable: bool,
		default_strategy: AllocationStrategy,
		other: T
	) -> Option<(NonNull<Self>, NonNull<T>)>
	{
		use bindings::region::zerOS_region_allocator_create;
		let maybe_inner = unsafe {
			zerOS_region_allocator_create(
				region.as_mut_ptr(),
				region.len(),
				static_memory,
				reclaimable,
				default_strategy.to_binding_strategy(),
				reclaim_hook
			)
		};
		if unlikely(maybe_inner.is_null())
		{
			return None;
		}
		unsafe {
			let mut default = Self {
				inner: maybe_inner,
				..Default::default()
			};
			let (pointers, remaining) = default.get_self_ptr();
			if let Some((self_ptr, after_self)) = pointers
			{
				default.self_storage = self_ptr;
				default.after_self = after_self;
				default.after_size = remaining;
				default.write_self_and_other_to_additional_storage(other)
			}
			else
			{
				None
			}
		}
	}

	unsafe fn get_additional_space(&mut self) -> (NonNull<u8>, usize)
	{
		use bindings::region::{
			zerOS_region_allocator_additional_space,
			zerOS_region_allocator_additional_space_info
		};

		let zerOS_region_allocator_additional_space_info { addr, size } =
			unsafe { zerOS_region_allocator_additional_space(self.inner) };

		(unsafe { NonNull::new_unchecked(addr) }, size)
	}

	unsafe fn get_self_ptr(&mut self) -> (Option<(NonNull<Self>, NonNull<u8>)>, usize)
	{
		let layout = Layout::for_value(self);
		let (addr, size) = unsafe { self.get_additional_space() };

		let alignment_padding = addr.align_offset(layout.align());

		if size < layout.size() + alignment_padding
		{
			(None, size)
		}
		else
		{
			unsafe {
				(
					Some((
						addr.byte_add(alignment_padding).cast(),
						addr.byte_add(alignment_padding).byte_add(layout.size())
					)),
					size - alignment_padding - layout.size()
				)
			}
		}
	}

	unsafe fn write_self_and_other_to_additional_storage<T>(
		self,
		other: T
	) -> Option<(NonNull<Self>, NonNull<T>)>
	{
		let layout = Layout::for_value(&other);
		let addr = self.after_self;
		let size = self.after_size;
		let storage = unsafe { self.write_self_to_additional_storage() };

		let alignment_padding = addr.align_offset(layout.align());

		if size < layout.size() + alignment_padding
		{
			None
		}
		else
		{
			unsafe {
				let returned = addr.byte_add(alignment_padding).cast();
				returned.write(other);
				Some((storage, returned))
			}
		}
	}

	unsafe fn write_self_to_additional_storage(self) -> NonNull<Self>
	{
		unsafe {
			let storage = self.self_storage;
			storage.write(self);
			storage
		}
	}

	pub unsafe fn reclaim(&mut self) -> bool
	{
		use bindings::region::zerOS_region_allocator_reclaim;
		unsafe { zerOS_region_allocator_reclaim(self.inner) }
	}

	pub unsafe fn alloc_raw(&self, size: usize) -> *mut core::ffi::c_void
	{
		use bindings::region::zerOS_region_allocator_alloc;
		unsafe {
			zerOS_region_allocator_alloc(
				self.inner,
				size,
				usize::MAX,
				AllocationStrategy::Default.to_binding_strategy()
			)
		}
	}

	pub unsafe fn dealloc_raw(&self, ptr: *mut core::ffi::c_void)
	{
		use bindings::region::zerOS_region_allocator_dealloc;
		unsafe { zerOS_region_allocator_dealloc(self.inner, ptr) }
	}

	pub unsafe fn realloc_raw(
		&self,
		ptr: *mut core::ffi::c_void,
		size: usize
	) -> *mut core::ffi::c_void
	{
		use bindings::region::zerOS_region_allocator_realloc;
		unsafe {
			zerOS_region_allocator_realloc(
				self.inner,
				ptr,
				usize::MAX,
				usize::MAX,
				size,
				usize::MAX,
				AllocationStrategy::Default.to_binding_strategy()
			)
		}
	}

	pub fn allocate_with_strategy(
		&self,
		layout: Layout,
		strategy: AllocationStrategy
	) -> Result<NonNull<[u8]>, AllocError>
	{
		use bindings::region::zerOS_region_allocator_alloc;
		let ptr: *mut c_void = unsafe {
			zerOS_region_allocator_alloc(
				self.inner,
				layout.size(),
				layout.align(),
				strategy.to_binding_strategy()
			)
		};
		match !ptr.is_null()
		{
			true =>
			{
				Ok(unsafe {
					NonNull::slice_from_raw_parts(NonNull::new_unchecked(ptr.cast()), layout.size())
				})
			},
			_ => Err(AllocError)
		}
	}

	pub fn reallocate_with_strategy(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout,
		strategy: AllocationStrategy
	) -> Result<NonNull<[u8]>, AllocError>
	{
		use bindings::region::zerOS_region_allocator_realloc;
		let new: *mut c_void = unsafe {
			zerOS_region_allocator_realloc(
				self.inner,
				ptr.as_ptr().cast(),
				old_layout.size(),
				old_layout.align(),
				new_layout.size(),
				new_layout.align(),
				strategy.to_binding_strategy()
			)
		};
		match !new.is_null()
		{
			true =>
			{
				Ok(unsafe {
					NonNull::slice_from_raw_parts(
						NonNull::new_unchecked(new.cast()),
						new_layout.size()
					)
				})
			},
			_ => Err(AllocError)
		}
	}
}

unsafe impl Allocator for RegionAllocator
{
	fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError>
	{
		self.allocate_with_strategy(layout, AllocationStrategy::Default)
	}

	unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout)
	{
		use bindings::region::zerOS_region_allocator_dealloc;
		let _ = layout;
		unsafe { zerOS_region_allocator_dealloc(self.inner, ptr.as_ptr().cast()) }
	}

	unsafe fn grow(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout
	) -> Result<NonNull<[u8]>, AllocError>
	{
		debug_assert!(
			new_layout.size() >= old_layout.size(),
			"`new_layout.size()` must be greater than or equal to `old_layout.size()`"
		);
		self.reallocate_with_strategy(ptr, old_layout, new_layout, AllocationStrategy::Default)
	}

	unsafe fn shrink(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout
	) -> Result<NonNull<[u8]>, AllocError>
	{
		debug_assert!(
			new_layout.size() <= old_layout.size(),
			"`new_layout.size()` must be less than or equal to `old_layout.size()`"
		);
		self.reallocate_with_strategy(ptr, old_layout, new_layout, AllocationStrategy::Default)
	}

	unsafe fn grow_zeroed(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout
	) -> Result<NonNull<[u8]>, AllocError>
	{
		match unsafe { self.grow(ptr, old_layout, new_layout) }
		{
			Ok(ref mut new) =>
			{
				let zeroed_size = new_layout.size() - old_layout.size();
				unsafe {
					let write_at = new.as_mut_ptr().byte_add(old_layout.size());
					core::ptr::write_bytes(write_at, 0, zeroed_size);
				}
				Ok(*new)
			},
			other => other
		}
	}
}
