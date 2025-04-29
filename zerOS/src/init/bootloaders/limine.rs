use limine::{
	BaseRevision,
	request::{FramebufferRequest, HhdmRequest, MemoryMapRequest}
};

macro_rules! requests {
    {$($it:item)*} => {
        $(
            #[used]
            #[unsafe(link_section = ".requests")]
            $it
        )*
    };
}

requests! {
	pub static BASE_REVISION: BaseRevision = BaseRevision::new();
	pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();
	pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
	pub static MEMMAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();
}

mod __markers
{
	use limine::request::{RequestsEndMarker, RequestsStartMarker};

	#[used]
	#[unsafe(link_section = ".requests_start_marker")]
	pub static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
	#[used]
	#[unsafe(link_section = ".requests_end_marker")]
	pub static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();
}

mod entry
{
	use super::*;
	use crate::{
		arch::target::cpu::misc::hcf,
		init::{self, ctors::CtorIter},
		kernel,
		kmain
	};

	#[unsafe(no_mangle)]
	extern "C" fn zerOS_boot_setup() -> !
	{
		// All limine requests must also be referenced in a called function, otherwise
		// they may be removed by the linker.
		assert!(BASE_REVISION.is_supported());

		CtorIter::new().for_each(|ctor| unsafe { ctor() });

		let under_qemu = kernel::hypervisor::under_qemu();
		if under_qemu.is_err() || !under_qemu.expect("unreachable")
		{
			hcf();
		}

		init::memory::gdt::init();

		kmain()
	}
}
