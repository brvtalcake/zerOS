use core::arch::asm;

pub fn enable()
{
    unsafe {
        asm! {
            "sti",
            options(att_syntax),
            options(nomem),
            options(nostack)
        }
    }
}

pub fn disable()
{
    unsafe {
        asm! {
            "cli",
            options(att_syntax),
            options(nomem),
            options(nostack)
        }
    }
}
