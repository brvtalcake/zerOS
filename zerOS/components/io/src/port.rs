use core::{any::Any, ops::DerefMut};

use downcast_rs::{impl_downcast, Downcast};
use zerOS_utils::VoidResult;

use crate::KernelIOTypes;

pub trait KernelPortInput: KernelIOTypes + Downcast
{
	fn port_read(&mut self, read: &mut dyn Any) -> VoidResult<Self::Error>;

	fn port_read_multiple(&mut self, read: &mut [&mut dyn Any])
	-> VoidResult<(Self::Error, usize)>
	{
		let mut successful = 0;
		for el in read.iter_mut().map(DerefMut::deref_mut)
		{
			self.port_read(el).map_err(|err| (err, successful))?;
			successful += 1;
		}
		Ok(())
	}
}
impl_downcast!(KernelPortInput);

pub trait KernelPortOutput: KernelIOTypes + Downcast
{
	fn port_write(&mut self, written: &dyn Any) -> VoidResult<Self::Error>;
}
impl_downcast!(KernelPortOutput);

mod tests
{
	use core::fmt::Display;

	use downcast_rs::impl_downcast;

	use crate::{KernelIOTypes, KernelPortInput};

	struct Test;

	#[derive(Debug)]
	struct ErrType;

	impl Display for ErrType
	{
		fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
		{
			Ok(())
		}
	}

	impl core::error::Error for ErrType {}

	impl KernelIOTypes for Test
	{
		type Error = ErrType;
	}

	impl KernelPortInput for Test
	{
		fn port_read(
			&mut self,
			read: &mut dyn core::any::Any
		) -> zerOS_utils::VoidResult<Self::Error>
		{
			Err(ErrType)
		}
	}
}
