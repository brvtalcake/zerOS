#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(variant_count)]

use cfg_if::cfg_if;
use zerOS_static_assertions::static_assert;

// sanity check
static_assert!(
	cfg!(x86_alike)
		|| cfg!(avr_alike)
		|| cfg!(sparc_alike)
		|| cfg!(loongarch_alike)
		|| cfg!(mips_alike)
		|| cfg!(ppc_alike)
		|| cfg!(riscv_alike)
		|| cfg!(arm_alike)
);

// sibling architectures common modules
cfg_if! {
	if #[cfg(x86_alike)]
	{
		mod x86_common;
	}
}

// per-arch modules
cfg_if! {
	if #[cfg(target_arch = "x86")] {
		pub mod x86;
		pub use self::x86 as target;
		pub use core::arch::x86 as core_target;
	} else if #[cfg(target_arch = "x86_64")] {
		pub mod amd64;
		pub use self::amd64 as x86_64; /// Convenient alias
		pub use self::amd64 as target;
		pub use core::arch::x86_64 as core_target;
	} else if #[cfg(target_arch = "aarch64")] {
		pub mod aarch64;
		pub use self::aarch64 as target;
		pub use core::arch::aarch64 as core_target;
	}
}

// per-arch types
cfg_if! {
	if #[cfg(target_arch = "x86")] {
		#[allow(non_camel_case_types)]
		pub type ureg = u32;
		#[allow(non_camel_case_types)]
		pub type ireg = i32;
	} else if #[cfg(target_arch = "x86_64")] {
		#[allow(non_camel_case_types)]
		pub type ureg = u64;
		#[allow(non_camel_case_types)]
		pub type ireg = i64;
	} else if #[cfg(target_arch = "aarch64")] {
		#[allow(non_camel_case_types)]
		pub type ureg = u64;
		#[allow(non_camel_case_types)]
		pub type ireg = i64;
	}
}
