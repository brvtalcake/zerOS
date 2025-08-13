use core::any::Any;

use downcast_rs::{Downcast, impl_downcast};
use zerOS_utils::VoidResult;

use crate::{IOError, KernelInput, KernelOutput};

pub trait KernelPortInput: KernelInput
{
	fn port_read(&mut self, read: &mut dyn Any) -> VoidResult<IOError>;

	fn port_read_multiple(
		&mut self,
		read: &mut dyn Iterator<Item = &mut dyn Any>
	) -> VoidResult<(IOError, usize)>
	{
		let mut successful = 0;
		for el in read
		{
			self.port_read(el).map_err(|err| (err, successful))?;
			successful += 1;
		}
		Ok(())
	}
}
impl_downcast!(sync KernelPortInput);

pub trait KernelPortOutput: KernelOutput
{
	fn port_write(&mut self, written: &dyn Any) -> VoidResult<IOError>;
}
impl_downcast!(sync KernelPortOutput);

pub trait KernelPortIO: KernelPortInput + KernelPortOutput {}
impl_downcast!(sync KernelPortIO);

impl<T: KernelPortInput + KernelPortOutput> KernelPortIO for T {}
