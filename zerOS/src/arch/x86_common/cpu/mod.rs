use overloadable::overloadable;
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        pub use core::arch::x86_64::CpuidResult;
        use core::arch::x86_64::__cpuid_count;
    } else if #[cfg(target_arch = "x86")] {
        pub use core::arch::x86::CpuidResult;
        use core::arch::x86::__cpuid_count;
    } else {
        compile_error!("should be unreachable !");
    }
}

pub mod io;
pub mod irq;
pub mod misc;
pub mod msr;

overloadable! {
	pub cpuid as

	fn(leaf: u32, subleaf: u32) -> CpuidResult
	{
		unsafe {
			__cpuid_count(leaf, subleaf)
		}
	},
	fn(leaf: u32) -> CpuidResult
	{
		unsafe {
			__cpuid_count(leaf, 0)
		}
	}
}
