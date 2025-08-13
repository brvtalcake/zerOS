use core::mem;

use elain::{Align, Alignment};
use zerocopy::TryFromBytes;

use crate::Aligned;

#[derive(Debug)]
pub struct AlignedBuffer<T: TryFromBytes>(
	Aligned<{ mem::align_of::<T>() }, [u8; mem::size_of::<T>()]>
)
where
	Align<{ mem::align_of::<T>() }>: Alignment,
	[u8; mem::size_of::<T>()]: Default;

impl<T: TryFromBytes> const Default for AlignedBuffer<T>
where
	Align<{ mem::align_of::<T>() }>: Alignment,
	[u8; mem::size_of::<T>()]: [const] Default
{
	fn default() -> Self
	{
		Self(Aligned::default())
	}
}

impl<T: TryFromBytes> AlignedBuffer<T>
where
	Align<{ mem::align_of::<T>() }>: Alignment,
	[u8; mem::size_of::<T>()]: Default
{
	pub const fn new() -> Self
	{
		Self(Default::default())
	}
}
