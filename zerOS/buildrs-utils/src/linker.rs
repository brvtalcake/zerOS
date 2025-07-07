use std::path::PathBuf;

use crate::{cargo::get_target_arch, get_topdir};

pub fn linker_script(template: bool) -> PathBuf
{
	let arch = get_target_arch().expect("could not determine zerOS target architecture");
	get_topdir().join("zerOS").join("linker").join(format!(
		"linker-{arch}.ld{maybe_suffix}",
		maybe_suffix = template.then_some(".template").unwrap_or("")
	))
}
