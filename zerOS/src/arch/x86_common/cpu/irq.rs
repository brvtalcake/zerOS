use core::arch::asm;

#[inline]
pub fn enable()
{
	unsafe {
		asm! {
			"sti",
			options(att_syntax, nomem, nostack)
		}
	}
}
#[inline]
pub fn disable()
{
	unsafe {
		asm! {
			"cli",
			options(att_syntax, nomem, nostack)
		}
	}
}
