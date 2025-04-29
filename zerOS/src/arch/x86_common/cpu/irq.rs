use core::arch::asm;

#[inline]
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
#[inline]
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
