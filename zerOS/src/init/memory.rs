pub mod gdt
{
	use lazy_static::lazy_static;

	use crate::kernel;

	lazy_static! {
		static ref GDT: kernel::memory::gdt::GDT = kernel::memory::gdt::GDT::default();
	}

	pub fn init()
	{
		unsafe {
			GDT.set();
		}
	}
}
