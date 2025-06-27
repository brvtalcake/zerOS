use std::ffi::OsStr;

use camino::Utf8PathBuf;
use tokio::process;

use crate::{
	SupportedArch,
	actions::configure::{config_location, get_topdir},
	tools::CmdIn
};

pub(crate) fn generate_target_default(
	cargo: impl AsRef<OsStr>,
	arch: SupportedArch,
	cpu: &String
) -> (CmdIn, Utf8PathBuf)
{
	match arch
	{
		SupportedArch::Amd64 =>
		{
			let generated = generate_target_default_amd64(cargo, cpu);
			(
				CmdIn::new(&get_topdir().join("generate-target"), generated.0),
				generated.1
			)
		},
		other =>
		{
			todo!(
				"supported for \"{}\" is not implemented yet",
				other.as_ref()
			)
		}
	}
}

fn generate_target_default_amd64(
	cargo: impl AsRef<OsStr>,
	cpu: &String
) -> (process::Command, Utf8PathBuf)
{
	let outfile = config_location!().join("x86_64-unknown-kernel.json");
	let mut cmd = process::Command::new(cargo);
	cmd.args([
		"run",
		"--release",
		"--",
		"--debug",
		"--arch=amd64",
		format!("--cpu={cpu}").as_str(),
		"--override=all",
		//"--rustc-abi=x86-sse2",
		"--frame-pointer=never",
		outfile.as_str()
	]);
	(cmd, outfile)
}
