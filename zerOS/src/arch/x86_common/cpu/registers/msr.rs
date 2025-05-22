use core::arch::asm;

pub fn read(msr: u32) -> u64
{
	let (high, low): (u32, u32);
	unsafe {
		asm! {
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
		asm! {
			"wrmsr",
			in("ecx") msr,
			in("eax") low,
			in("edx") high,
			options(att_syntax, nomem, nostack)
		};
	}
}
