use overloadable::overloadable;

pub mod cpu;
pub mod hypervisor;
pub mod linker;
pub mod memory;
pub mod serial;
pub mod sync;

pub use core::arch::x86_64::CpuidResult;

overloadable! {
    pub cpuid as

    fn(leaf: u32, subleaf: u32) -> CpuidResult
    {
        unsafe {
            core::arch::x86_64::__cpuid_count(leaf, subleaf)
        }
    },
    fn(leaf: u32) -> CpuidResult
    {
        unsafe {
            core::arch::x86_64::__cpuid_count(leaf, 0)
        }
    }
}
