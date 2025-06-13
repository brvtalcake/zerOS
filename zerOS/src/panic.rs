use alloc::format;
use core::sync::atomic::Ordering;

use portable_atomic::AtomicBool;

use crate::{error, unwinding};

static RUNNING_PANIC: AtomicBool = AtomicBool::new(false);

#[panic_handler]
fn rust_panic_impl(info: &core::panic::PanicInfo) -> !
{
	// let regs = unwinding::read_registers!();
	let mut line_buf = itoa::Buffer::new();
	let mut column_buf = itoa::Buffer::new();
	error!(
		r#"
PANIC at {}{}{}:
	{}
"#,
		info.location().map_or("<unknown-file>", |loc| loc.file()),
		info.location()
			.map_or("<unknown-line>", |loc| line_buf.format(loc.line())),
		info.location()
			.map_or("<unknown-column>", |loc| column_buf.format(loc.column())),
		format!("{}", info.message()).replace("\n", "\n\t")
	);
	if RUNNING_PANIC.swap(true, Ordering::AcqRel)
	{
		error!("attempted to `panic!` while panicking. aborting kernel...");
	}
	else
	{
		error!("a panic has been triggered");
		todo!("unwind the stack ? or at the very least print a decent backtrace");
	}
	crate::arch::target::cpu::misc::hcf();
	// TODO: print a stack trace
}
