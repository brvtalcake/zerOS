use std::{
	ffi::{OsStr, OsString},
	str::FromStr,
	sync::Arc
};

use anyhow::anyhow;
use clap::{Subcommand, ValueEnum};
use tokio::process;

use crate::{
	IntoArray,
	SupportedArch,
	XtaskGlobalOptions,
	actions::{
		Xtask,
		configure::{
			Executable,
			KConfigBootBootloader,
			ZerosConfig,
			get_default_executable_short_name,
			kcfg_write,
			subproj_location
		}
	},
	doc_comments::subdir,
	tools::{
		CmdIn,
		check,
		gentarget::generate_target_default,
		mk_iso,
		mkdir,
		rm,
		strip::{self, StripFlavor}
	}
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Subcommand)]
#[clap(rename_all = "lowercase")]
pub(crate) enum XtaskBuildableSubproj
{
	#[doc = subdir!(zerOS)]
	#[clap(name = "zerOS", alias("zeros"), rename_all = "kebab-case")]
	#[clap(about = subdir!(zerOS))]
	Zeros
	{
		#[arg(short, long, value_enum, default_value_t)]
		/// The architecture to build zerOS for
		arch:    SupportedArch,
		#[arg(short, long, value_enum, default_value_t)]
		/// The profile to build zerOS with
		profile: ZerosBuildProfile,
		#[arg(short, long, default_value = "native", alias("mcu"))]
		/// The CPU/MCU targetted by zerOS (alias: --mcu)
		cpu:     String
	},

	#[doc = subdir!(unwindtool)]
	#[clap(alias("unwind-tool"))]
	#[clap(about = subdir!(unwindtool))]
	UnwindTool,

	#[doc = subdir!(docs)]
	#[clap(about = subdir!(docs))]
	Docs
}

#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum, Default, strum::Display, strum::AsRefStr,
)]
#[clap(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum ZerosBuildProfile
{
	Dev,
	DevLTO,
	Release,
	#[default]
	ReleaseLTO
}

impl XtaskBuildableSubproj
{
	#[allow(non_snake_case)]
	async fn execute_zerOS(&self, globals: &XtaskGlobalOptions)
	{
		let Self::Zeros { arch, profile, cpu } = self
		else
		{
			unreachable!()
		};
		unsafe {
			std::env::set_var("ZEROS_TARGET_CPU", cpu);
			std::env::set_var("ZEROS_TARGET_ARCH", arch.as_ref());
			std::env::set_var("ZEROS_PROFILE", profile.as_ref());
		}

		// load cfg
		let cfg = Arc::new(ZerosConfig::load_or_error());
		let cloned_cfg = cfg.clone();
		let kcfg_write = tokio::task::spawn_blocking(move || kcfg_write(&cloned_cfg.kcfg));

		// generate target
		let (cmd, json_target) = generate_target_default(cfg.get(&Executable::Cargo), *arch, cpu);
		cmd.finalize().await;

		// prepare output directories
		let _ = tokio::join!(
			kcfg_write,
			tokio::spawn(async { rm(true, false, &subproj_location!("zerOS").join("bin")).await }),
			tokio::spawn(async {
				rm(true, false, &subproj_location!("zerOS").join("iso-root")).await
			}),
		)
		.into_array()
		.map(|res| check!(res.expect("failed to run tokio task")));
		let _ = tokio::join!(
			tokio::spawn(async {
				mkdir(
					true,
					false,
					&subproj_location!("zerOS")
						.join("bin")
						.join("zerOS-boot-modules")
				)
				.await
			}),
			tokio::spawn(async {
				mkdir(true, false, &subproj_location!("zerOS").join("iso-root")).await
			})
		)
		.into_array()
		.map(|res| check!(res.expect("failed to run tokio task")));

		let mut cmd = process::Command::new(cfg.get(&Executable::Cargo));
		cmd.args(&[
			"build",
			format!("--target={json_target}").as_str(),
			"-Z",
			"unstable-options",
			"--artifact-dir",
			"./bin",
			format!("--profile={profile}").as_str()
		])
		.env(
			"RUSTFLAGS",
			format!(
				"--cfg getrandom_backend=\"rdrand\" -Cforce-unwind-tables -Zmacro-backtrace \
				 -Ctarget-cpu={}",
				std::env::var("ZEROS_TARGET_CPU").unwrap()
			)
		);
		let cmd = CmdIn::new(&subproj_location!("zerOS"), cmd);
		cmd.finalize().await;

		let alt_strips = [Executable::Strip, Executable::EuStrip]
			.map(|e| "`".to_owned() + get_default_executable_short_name(&e) + "`");
		let alt_str =
			alt_strips[0..(alt_strips.len() - 1)].join(", ") + " or " + alt_strips.last().unwrap();
		let zeros_bin = subproj_location!("zerOS").join("bin");
		let mut objcopy = None;
		strip::run(
			check!(
				cfg.executables
					.get(&Executable::EuStrip)
					.or_else(|| {
						let strip = cfg.executables.get(&Executable::Strip)?;
						objcopy = Some(cfg.executables.get(&Executable::Objcopy)?);
						Some(strip)
					})
					.ok_or(anyhow!(
						"none of the following executables could be found: {alt_str}"
					))
					.expect("could not strip kernel binary")
			),
			objcopy.map_or(StripFlavor::ElfUtils, |obj| {
				StripFlavor::Unix { objcopy: obj }
			}),
			zeros_bin.join("zerOS"),
			zeros_bin.join("zerOS.stripped"),
			matches!(profile, ZerosBuildProfile::Dev | ZerosBuildProfile::DevLTO)
				.then(|| zeros_bin.join("zerOS-boot-modules").join("debug-info.zko"))
		)
		.await;

		mk_iso::run(
			zeros_bin.join("zerOS.stripped"),
			zeros_bin.join("zerOS.iso"),
			subproj_location!("zerOS").join("iso-root"),
			cfg.kcfg.boot.bootloader,
			*arch,
			zeros_bin.join("zerOS-boot-modules"),
			match cfg.kcfg.boot.bootloader
			{
				KConfigBootBootloader::Limine =>
				{
					Some(
						subproj_location!("zerOS")
							.join("config")
							.join("limine.conf")
					)
				},
				_ => None
			}
		)
		.await;
	}
}

impl Xtask for XtaskBuildableSubproj
{
	async fn execute(&self, globals: &XtaskGlobalOptions)
	{
		match self
		{
			Self::Zeros { .. } => self.execute_zerOS(globals).await,
			_ => todo!()
		}
	}
}
