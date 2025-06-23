use std::ffi::OsStr;

use camino::Utf8PathBuf;

use crate::{
	SupportedArch,
	actions::{
		build::ZerosBuildProfile,
		configure::{config_location, get_topdir}
	},
	tools::CmdIn
};

pub(crate) fn generate_target_default(
	cargo: impl AsRef<OsStr>,
	arch: SupportedArch,
	cpu: String
) -> (CmdIn, Utf8PathBuf)
{
	assert_eq!(
		arch,
		SupportedArch::Amd64,
		"supported for architectures other than amd64 is not yet implemented"
	);
	let archstr = arch.as_ref();
	let outfile = config_location!().join(format!("{archstr}-unknown-kernel.json"));
	let mut cmd = std::process::Command::new(cargo);
	cmd.args([
		"run",
		"--release",
		"--",
		"--debug",
		format!("--arch={archstr}").as_str(),
		format!("--cpu={cpu}").as_str(),
		"--override=all",
		//"--rustc-abi=x86-sse2",
		"--frame-pointer=never",
		outfile.as_str()
	]);
	(
		CmdIn::new(&get_topdir().join("generate-target"), cmd),
		outfile
	)
}
