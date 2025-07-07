use std::{ffi::OsStr, sync::OnceLock};

#[macro_export]
macro_rules! to_cargo {
	($cfgstring:expr => $what:expr) => {
		println!("cargo::{}={}", $cfgstring, $what)
	};
}

#[macro_export]
macro_rules! from_cargo {
	($cfgvar:expr) => {
		::std::env::var_os($cfgvar)
	};
}

pub fn get_outdir() -> Option<&'static String>
{
	static OUTDIR: OnceLock<Option<String>> = OnceLock::new();
	OUTDIR
		.get_or_init(|| from_cargo!("OUT_DIR").map(|val| val.into_string().unwrap()))
		.as_ref()
}

/// returns target triple
pub fn get_target_triple() -> Option<&'static String>
{
	static TARGET: OnceLock<Option<String>> = OnceLock::new();
	TARGET
		.get_or_init(|| from_cargo!("TARGET").map(|val| val.into_string().unwrap()))
		.as_ref()
}

pub fn get_target_arch() -> Option<&'static String>
{
	static ARCH: OnceLock<Option<String>> = OnceLock::new();
	ARCH.get_or_init(|| from_cargo!("CARGO_CFG_TARGET_ARCH").map(|val| val.into_string().unwrap()))
		.as_ref()
}

pub fn get_opt_lvl() -> Option<&'static isize>
{
	static OPTLVL: OnceLock<Option<isize>> = OnceLock::new();
	OPTLVL
		.get_or_init(|| {
			from_cargo!("OPT_LEVEL").map(|val| val.into_string().unwrap().parse().unwrap())
		})
		.as_ref()
}

pub fn get_profile() -> Option<&'static String>
{
	static PROFILE: OnceLock<Option<String>> = OnceLock::new();
	PROFILE
		.get_or_init(|| from_cargo!("PROFILE").map(|val| val.into_string().unwrap()))
		.as_ref()
}

pub fn get_target_cpu() -> Option<&'static String>
{
	static TARGET_CPU: OnceLock<Option<String>> = OnceLock::new();
	TARGET_CPU
		.get_or_init(|| from_cargo!("ZEROS_TARGET_CPU").map(|val| val.into_string().unwrap()))
		.as_ref()
}

pub fn get_target_ptr_width() -> Option<&'static usize>
{
	static TARGET_POINTER_WIDTH: OnceLock<Option<usize>> = OnceLock::new();
	TARGET_POINTER_WIDTH
		.get_or_init(|| {
			from_cargo!("CARGO_CFG_TARGET_POINTER_WIDTH")
				.map(|val| val.into_string().unwrap().parse().unwrap())
		})
		.as_ref()
}

pub fn feature_activated(feature: impl AsRef<OsStr>) -> bool
{
	from_cargo!(feature).is_some()
}
