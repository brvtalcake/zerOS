use core::{cell::UnsafeCell, ptr::NonNull};

use super::allocators::{AllocationStrategy, RegionAllocator, RegionAllocatorReclaimHook};
use crate::kernel::sync::BasicMutex;

struct RegionAllocatorListLink
{
	prev:  Option<NonNull<RegionAllocatorListLink>>,
	next:  Option<NonNull<RegionAllocatorListLink>>,
	value: Option<NonNull<RegionAllocator>>
}

struct RegionAllocatorList
{
	head:         Option<NonNull<RegionAllocatorListLink>>,
	tail:         Option<NonNull<RegionAllocatorListLink>>,
	last_visited: Option<NonNull<RegionAllocatorListLink>>
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

	fn add(
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
			self.last_visited.is_some(),
			"the `last_visited` field of `RegionAllocatorList` should always be \
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

pub struct KernelAllocator
{
	regions: BasicMutex<RegionAllocatorList>
}

#[overloadf::overload]
impl KernelAllocator
{
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
				head:         Some(new_link),
				tail:         Some(new_link),
				last_visited: Some(new_link)
			})
		})
	}

	pub fn add_region_basic(&self, region: &'static mut [u8], static_memory: bool) -> Option<&Self>
	{
		unsafe { self.add_region(region, static_memory, AllocationStrategy::BestFit) }
	}

	pub fn add_region(
		&self,
		region: &'static mut [u8],
		static_memory: bool,
		default_strategy: AllocationStrategy
	) -> Option<&Self>
	{
		unsafe { self.add_region_extended(region, static_memory, None, false, default_strategy) }
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
		self.regions.lock().add(
			region,
			static_memory,
			reclaim_hook,
			reclaimable,
			default_strategy
		)?;
		Some(self)
	}
}
