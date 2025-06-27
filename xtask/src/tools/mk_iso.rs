use camino::Utf8Path;
use tokio::process;

use crate::{
	SupportedArch,
	actions::configure::{KConfigBootBootloader, subproj_location},
	tools::{CmdIn, check, check_opt}
};

/// TODO: the script shall accept a command-line parameter to specify the
/// `xorriso` binary
///
/// TODO: alternatively, do everything from rust !
pub(crate) fn run_in(
	pwd: impl AsRef<Utf8Path>,
	infile: impl AsRef<str>,
	outfile: impl AsRef<str>,
	iso_root: impl AsRef<str>,
	bootloader: KConfigBootBootloader,
	arch: SupportedArch,
	boot_modules_dir: impl AsRef<str>,
	bootloader_config: Option<impl AsRef<str>>
) -> CmdIn
{
	let mut cmd = process::Command::new(subproj_location!("scripts").join("mk_iso.py"));
	cmd.args(&[
		"-o",
		outfile.as_ref(),
		"--iso-root",
		iso_root.as_ref(),
		"-b",
		bootloader.as_ref(),
		"-a",
		arch.as_ref(),
		"-m",
		boot_modules_dir.as_ref()
	]);
	if let Some(bcfg) = bootloader_config
	{
		cmd.args(&["-c", bcfg.as_ref()]);
	}
	cmd.arg(infile.as_ref());
	CmdIn::new(pwd, cmd)
}

pub(crate) async fn run(
	infile: impl AsRef<str>,
	outfile: impl AsRef<str>,
	iso_root: impl AsRef<str>,
	bootloader: KConfigBootBootloader,
	arch: SupportedArch,
	boot_modules_dir: impl AsRef<str>,
	bootloader_config: Option<impl AsRef<str>>
)
{
	run_in(
		check_opt!(
			Utf8Path::from_path(&check!(
				std::env::current_dir().expect("could not retrieve current working directory")
			))
			.expect("could not create a valid UTF-8 path")
		),
		infile,
		outfile,
		iso_root,
		bootloader,
		arch,
		boot_modules_dir,
		bootloader_config
	)
	.finalize()
	.await;
}
