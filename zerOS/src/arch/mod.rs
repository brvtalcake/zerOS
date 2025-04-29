use cfg_if::cfg_if;

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
	} else if #[cfg(target_arch = "x86_64")] {
		pub mod amd64;
		pub use self::amd64 as x86_64; /// Convenient alias
		pub use self::amd64 as target;
	} else if #[cfg(target_arch = "aarch64")] {
		pub mod arm64;
		pub use self::arm64 as target;
	}
}