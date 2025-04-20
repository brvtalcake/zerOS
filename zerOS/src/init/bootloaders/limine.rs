use limine::BaseRevision;
use limine::request::{FramebufferRequest, MemoryMapRequest, HhdmRequest};

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
