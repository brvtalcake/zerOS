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
use core::ptr::NonNull;

mod bindings;

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

pub struct RegionAllocator
{
	inner: *mut bindings::region::zerOS_region_allocator
}

impl RegionAllocator
{
	pub fn allocate_with_strategy(
		&self,
		layout: Layout,
		strategy: AllocationStrategy
	) -> Result<NonNull<[u8]>, AllocError>
	{
		let ptr: *mut u8 = unsafe {
			bindings::region::zerOS_region_allocator_alloc(
				self.inner,
				layout.size(),
				layout.align(),
				strategy.to_binding_strategy() as _
			)
		}
		.cast();
		match !ptr.is_null()
		{
			true =>
			{
				Ok(unsafe {
					NonNull::slice_from_raw_parts(NonNull::new_unchecked(ptr), layout.size())
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
		let _ = layout;
		unsafe { bindings::region::zerOS_region_allocator_dealloc(self.inner, ptr.as_ptr().cast()) }
	}

	unsafe fn grow(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout
	) -> Result<NonNull<[u8]>, AllocError>
	{
		todo!("just use realloc");
		if old_layout.align() != new_layout.align()
		{
			let new_storage = self.allocate(new_layout)?;

			// SAFETY: see the default trait implementation
			unsafe {
				core::ptr::copy_nonoverlapping(
					ptr.as_ptr(),
					new_storage.as_mut_ptr(),
					old_layout.size()
				);
				self.deallocate(ptr, old_layout);
			}

			Ok(new_storage)
		}
		else
		{
			todo!()
		}
	}

	unsafe fn shrink(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout
	) -> Result<NonNull<[u8]>, AllocError>
	{
		todo!()
	}

	fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError>
	{
		todo!()
	}

	unsafe fn grow_zeroed(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout
	) -> Result<NonNull<[u8]>, AllocError>
	{
		todo!()
	}
}
