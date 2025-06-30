use std::{ffi::OsStr, ops::Deref, sync::Arc};

use camino::{Utf8Path, Utf8PathBuf};
use tokio::{process, task};

use crate::{
	IntoArray,
	SupportedArch,
	actions::configure::{KConfigBootBootloader, subproj_location},
	tools::{CmdIn, check, check_opt, cp, mkdir}
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

pub(crate) async fn run_from_rust(
	xorriso: impl AsRef<OsStr>,
	infile: impl AsRef<Utf8Path>,
	outfile: impl AsRef<str>,
	iso_root: impl AsRef<Utf8Path>,
	bootloader: KConfigBootBootloader,
	arch: SupportedArch,
	boot_modules_dir: impl AsRef<Utf8Path>,
	bootloader_config: Option<impl AsRef<Utf8Path>>,
	other_args: impl IntoIterator<Item = impl AsRef<OsStr>>
)
{
	let root = Arc::new(iso_root.as_ref().to_path_buf());
	let inf = Arc::new(infile.as_ref().to_path_buf());
	let bootmods = Arc::new(boot_modules_dir.as_ref().to_path_buf());
	let bootconf = Arc::new(bootloader_config.map(|conf| conf.as_ref().to_path_buf()));
	match bootloader
	{
		KConfigBootBootloader::Limine =>
		{
			mkdir(true, false, &iso_root.as_ref().join("boot").join("limine")).await;
			let _ = tokio::join!(
				task::spawn((async move |inf: Arc<Utf8PathBuf>,
				                         root: Arc<Utf8PathBuf>| {
					cp(&inf, &root.join("boot").join(inf.file_name().unwrap())).await
				})(inf.clone(), root.clone())),
				task::spawn((async move |bootmods: Arc<Utf8PathBuf>,
				                         root: Arc<Utf8PathBuf>| {
					cp(
						&bootmods,
						&root.join("boot").join(bootmods.file_name().unwrap())
					)
					.await
				})(bootmods.clone(), root.clone())),
				task::spawn((async move |bootconf: Arc<Option<Utf8PathBuf>>,
				                         root: Arc<Utf8PathBuf>| {
					if let Some(conf) = &*bootconf
					{
						cp(
							&conf,
							&root
								.join("boot")
								.join("limine")
								.join(conf.file_name().unwrap())
						)
						.await
					}
				})(bootconf.clone(), root.clone()))
			)
			.into_array()
			.map(|res| check!(res.expect("failed to run tokio task")));
			CmdIn::new(
				check_opt!(
					Utf8Path::from_path(&check!(
						std::env::current_dir()
							.expect("could not retrieve current working directory")
					))
					.expect("could not create a valid UTF-8 path")
				),
				{
					let mut cmd = process::Command::new(xorriso);
					cmd.args(&[
						"-as",
						"mkisofs",
						"-no-emul-boot",
						"-boot-load-size",
						"4",
						"-boot-info-table",
						"-efi-boot-part",
						"--efi-boot-image",
						"--protective-msdos-label",
						iso_root.as_ref().as_ref(),
						"-o",
						outfile.as_ref()
					])
					.args(other_args);
					cmd
				}
			)
			.finalize()
			.await
		},
		_ => todo!()
	}
}
