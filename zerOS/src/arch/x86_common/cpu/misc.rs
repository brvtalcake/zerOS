use core::arch::asm;

use super::{io::outw, irq};

pub fn halt()
{
	unsafe {
		asm! {
			"hlt",
			options(att_syntax, nomem, nostack)
		}
	}
}

pub fn reboot() -> !
{
	outw(0x64, 0xfe | (1 << 8));
	halt();
	unreachable!();
}

/// Halt and catch fire
pub fn hcf() -> !
{
	loop
	{
		irq::disable();
		halt();
	}
}
