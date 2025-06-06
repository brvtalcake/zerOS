use core::sync::atomic::Ordering;

use portable_atomic::AtomicBool;

use crate::error;
use crate::unwinding;

static RUNNING_PANIC: AtomicBool = AtomicBool::new(false);

#[panic_handler]
fn rust_panic_impl(_info: &core::panic::PanicInfo) -> !
{
	/* let regs = unwinding::read_registers!(); */
	if RUNNING_PANIC.swap(true, Ordering::AcqRel)
	{
		error!("attempted to `panic!` while panicking");
	}
	else
	{
		error!("a panic has been triggered");
		todo!("unwind the stack ? or at the very least print a decent backtrace");
	}
	crate::arch::target::cpu::misc::hcf();
	// TODO: print a stack trace
}
