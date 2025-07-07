pub(super) mod extended
{
	pub mod xcr0
	{
		use crate::arch::core_target::{_xgetbv, _xsetbv};

		pub fn read() -> u64
		{
			unsafe { _xgetbv(0) }
		}

		pub fn write(value: u64)
		{
			unsafe {
				_xsetbv(0, value);
			}
		}
	}
}

pub mod cr0
{
	use core::arch::asm;

	use crate::arch::ureg;

	pub fn read() -> ureg
	{
		let mut result: ureg;
		unsafe {
			asm! {
				"mov %cr0 {res}",
				res = out(reg) result,
				options(att_syntax, nomem, nostack)
			}
		}
        result
	}

    pub fn write(value: ureg)
	{
		unsafe {
			asm! {
				"mov {val} %cr0",
				val = in(reg) value,
				options(att_syntax, nomem, nostack)
			}
		}
	}
}

pub mod cr3
{
	use core::arch::asm;

	use crate::arch::ureg;

	pub fn read() -> ureg
	{
		let mut result: ureg;
		unsafe {
			asm! {
				"mov %cr3 {res}",
				res = out(reg) result,
				options(att_syntax, nomem, nostack)
			}
		}
        result
	}

    pub fn write(value: ureg)
	{
		unsafe {
			asm! {
				"mov {val} %cr3",
				val = in(reg) value,
				options(att_syntax, nomem, nostack)
			}
		}
	}
}

pub mod cr4
{
	use core::arch::asm;

	use crate::arch::ureg;

	pub fn read() -> ureg
	{
		let mut result: ureg;
		unsafe {
			asm! {
				"mov %cr4 {res}",
				res = out(reg) result,
				options(att_syntax, nomem, nostack)
			}
		}
        result
	}

    pub fn write(value: ureg)
	{
		unsafe {
			asm! {
				"mov {val} %cr4",
				val = in(reg) value,
				options(att_syntax, nomem, nostack)
			}
		}
	}
}