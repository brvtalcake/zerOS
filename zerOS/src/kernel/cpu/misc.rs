use core::arch::asm;

use super::irq;

use crate::kernel::cpu::io::outw;

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
    outw(0x64, 0xFE | (1 << 8));
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
