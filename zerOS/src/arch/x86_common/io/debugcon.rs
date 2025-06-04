use lazy_static::lazy_static;

use crate::{
	arch::target,
	kernel::{
		hypervisor,
		io::{KernelIO, KernelOutput},
		logging,
		sync::BasicMutex
	}
};
pub struct DebugCon;

impl alloc::fmt::Write for DebugCon
{
	fn write_str(&mut self, s: &str) -> alloc::fmt::Result
	{
		self.write_bytes(s.as_bytes());
		Ok(())
	}
}

impl KernelOutput for DebugCon
{
	fn supports_ansi_escape_codes(&self) -> bool
	{
		false
	}

	fn flush(&mut self) -> core::fmt::Result
	{
		Ok(())
	}

	fn write_byte(&self, byte: u8)
	{
		target::cpu::io::immediate_outb::<0xe9>(byte);
	}
}

static ZEROS_DEBUGCON_LOGGER: BasicMutex<DebugCon> = BasicMutex::new(DebugCon);

ctor! {
	@priority(100);
	@name(zerOS_init_debugcon_logger);
	if hypervisor::under_qemu().unwrap()
	{
		logging::ZEROS_GLOBAL_LOGGER.add_logger(
			&ZEROS_DEBUGCON_LOGGER, None, logging::LoggingBackend::DebugCon
		).unwrap();
		logging::set_global_backend_state(logging::LoggingBackend::DebugCon, true);
	}
}
