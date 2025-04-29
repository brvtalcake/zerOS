use core::arch::asm;

use super::irq;
use super::io::outw;

pub fn halt()
{
	unsafe {
		asm! {
			"hlt",
			options(att_syntax),
			options(nomem),
			options(nostack)
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
