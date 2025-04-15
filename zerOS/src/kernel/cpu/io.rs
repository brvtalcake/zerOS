use core::arch::asm;

#[inline]
pub fn inb(port: u16) -> u8
{
    let ret: u8;
    unsafe {
        asm! {
            "inb {inport:x}, {outval}",
            inport = in(reg) port,
            outval = out(reg_byte) ret,
            options(att_syntax, nomem, nostack)
        };
    }
    ret
}

#[inline]
pub fn outb(port: u16, value: u8)
{
    unsafe {
        asm!{
            "outb {inport:x}, {inval}",
            inport = in(reg) port,
            inval  = in(reg_byte) value,
            options(att_syntax, nomem, nostack)
        }
    }
}

#[inline]
pub fn inw(port: u16) -> u16
{
    let ret: u16;
    unsafe {
        asm!{
            "inw {inport:x}, {outval:x}",
            inport = in(reg) port,
            outval = out(reg) ret,
            options(att_syntax, nomem, nostack)
        };
    }
    ret
}

#[inline]
pub fn outw(port: u16, value: u16)
{
    unsafe {
        asm!{
            "outw {inport:x}, {inval:x}",
            inport = in(reg) port,
            inval  = in(reg) value,
            options(att_syntax, nomem, nostack)
        }
    }
}

#[inline]
pub fn inl(port: u16) -> u32
{
    let ret: u32;
    unsafe {
        asm!{
            "inl {inport:x}, {outval:e}",
            inport = in(reg) port,
            outval = out(reg) ret,
            options(att_syntax, nomem, nostack)
        };
    }
    ret
}

#[inline]
pub fn outl(port: u16, value: u32)
{
    unsafe {
        asm!{
            "outw {inport:x}, {inval:e}",
            inport = in(reg) port,
            inval  = in(reg) value,
            options(att_syntax, nomem, nostack)
        }
    }
}

pub mod msr
{
    use core::arch::asm;

    pub fn read(msr: u32) -> u64
    {
        let (high, low): (u32, u32);
        unsafe {
            asm!{
                "rdmsr",
                out("eax") low,
                out("edx") high,
                in("ecx") msr,
                options(att_syntax, nomem, nostack)
            };
        }
        ((high as u64) << 32) | (low as u64)
    }

    pub fn write(msr: u32, value: u64)
    {
        let low = value as u32;
        let high = (value >> 32) as u32;
        unsafe {
            asm!{
                "wrmsr",
                in("ecx") msr,
                in("eax") low,
                in("edx") high,
                options(att_syntax, nomem, nostack)
            };
        }
    }
}
