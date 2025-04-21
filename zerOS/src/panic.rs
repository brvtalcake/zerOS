#[panic_handler]
fn rust_panic_impl(_info: &core::panic::PanicInfo) -> !
{
	crate::kernel::cpu::misc::hcf();
}
