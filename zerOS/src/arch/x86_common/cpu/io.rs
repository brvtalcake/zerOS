//! TODO: what about string versions ? (`ins` and `outs`)

use core::arch::asm;

#[inline]
pub fn inb(port: u16) -> u8
{
	let ret: u8;
	unsafe {
		asm! {
			"inb %dx, %al",
			in("dx") port,
			out("al") ret,
			options(att_syntax, nomem, nostack)
		};
	}
	ret
}

#[inline]
pub fn immediate_inb<const PORT: u8>() -> u8
{
	let ret: u8;
	unsafe {
		asm! {
			"inb {inport}, %al",
			inport = const PORT,
			out("al") ret,
			options(att_syntax, nomem, nostack)
		};
	}
	ret
}

#[inline]
pub fn outb(port: u16, value: u8)
{
	unsafe {
		asm! {
			"outb %al, %dx",
			in("dx") port,
			in("al") value,
			options(att_syntax, nomem, nostack)
		}
	}
}

#[inline]
pub fn immediate_outb<const PORT: u8>(value: u8)
{
	unsafe {
		asm! {
			"outb %al, ${inport}",
			inport = const PORT,
			in("al") value,
			options(att_syntax, nomem, nostack)
		}
	}
}

#[inline]
pub fn inw(port: u16) -> u16
{
	let ret: u16;
	unsafe {
		asm! {
			"inw %dx, %ax",
			in("dx") port,
			out("ax") ret,
			options(att_syntax, nomem, nostack)
		};
	}
	ret
}

#[inline]
pub fn outw(port: u16, value: u16)
{
	unsafe {
		asm! {
			"outw %ax, %dx",
			in("dx") port,
			in("ax") value,
			options(att_syntax, nomem, nostack)
		}
	}
}

#[inline]
pub fn inl(port: u16) -> u32
{
	let ret: u32;
	unsafe {
		asm! {
			"inl %dx, %eax",
			in("dx") port,
			out("eax") ret,
			options(att_syntax, nomem, nostack)
		};
	}
	ret
}

#[inline]
pub fn outl(port: u16, value: u32)
{
	unsafe {
		asm! {
			"outl %eax, %dx",
			in("dx") port,
			in("eax") value,
			options(att_syntax, nomem, nostack)
		}
	}
}