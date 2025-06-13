use gimli::X86;
#[derive(Debug, Default)]
pub struct RegisterSet
{
	pub eip: Option<u32>,
	pub esp: Option<u32>,
	pub ebp: Option<u32>,
	pub ret: Option<u32>
}

pub macro unwind_get_registers() {{
	let ip;
	let sp;
	let bp;
	let ret;
	unsafe {
		::core::arch::asm! {
			"mov %eip, {ip:e}",
			"mov %esp, {sp:e}",
			"mov %ebp, {bp:e}",
			ip = out(reg) ip,
			sp = out(reg) sp,
			bp = out(reg) bp,
			options(att_syntax)
		};
		ret = crate::llvm::return_address(0).addr() as u32;
	}
	crate::unwinding::RegisterSet {
		eip: Some(ip),
		esp: Some(sp),
		ebp: Some(bp),
		eet: Some(ret)
	}
}}
