use core::ops::{Deref, DerefMut};

pub struct Block<'bytes>
{
	view: &'bytes [u8]
}

impl<'bytes> Block<'bytes>
{
	pub fn new(region: &'bytes [u8]) -> Self
	{
		debug_assert!({
			let rawptr: *const u8 = &raw const region[0];
			rawptr.is_aligned_to(region.len())
		});
		Self { view: region }
	}
}

impl<'bytes> AsRef<[u8]> for Block<'bytes>
{
	fn as_ref(&self) -> &[u8]
	{
		self.view
	}
}

impl<'bytes> Deref for Block<'bytes>
{
	type Target = [u8];

	fn deref(&self) -> &Self::Target
	{
		<Self as AsRef<[u8]>>::as_ref(self)
	}
}

pub struct BlockMut<'bytes>
{
	view: &'bytes mut [u8]
}

impl<'bytes> BlockMut<'bytes>
{
	pub fn new(region: &'bytes mut [u8]) -> Self
	{
		debug_assert!({
			let rawptr: *const u8 = &raw const region[0];
			rawptr.is_aligned_to(region.len())
		});
		Self { view: region }
	}
}

impl<'bytes> AsRef<[u8]> for BlockMut<'bytes>
{
	fn as_ref(&self) -> &[u8]
	{
		self.view
	}
}

impl<'bytes> AsMut<[u8]> for BlockMut<'bytes>
{
	fn as_mut(&mut self) -> &mut [u8]
	{
		self.view
	}
}

impl<'bytes> Deref for BlockMut<'bytes>
{
	type Target = [u8];

	fn deref(&self) -> &Self::Target
	{
		<Self as AsRef<[u8]>>::as_ref(self)
	}
}

impl<'bytes> DerefMut for BlockMut<'bytes>
{
	fn deref_mut(&mut self) -> &mut <Self as Deref>::Target
	{
		<Self as AsMut<[u8]>>::as_mut(self)
	}
}
