use clap::{Subcommand, ValueEnum};

use crate::{
	SupportedArch,
	XtaskGlobalOptions,
	actions::{
		Xtask,
		configure::{Executable, ZerosConfig, subproj_location}
	},
	doc_comments::subdir,
	tools::{CmdIn, gentarget::generate_target_default, mkdir, rm}
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
	fn execute_zerOS(&self, globals: &XtaskGlobalOptions)
	{
		let Self::Zeros { arch, profile, cpu } = self
		else
		{
			unreachable!()
		};

		// load cfg
		let cfg = ZerosConfig::load_or_error();

		// generate target
		let (cmd, json_target) = generate_target_default(cfg.get(&Executable::Cargo), *arch, cpu);
		cmd.finalize();

		// prepare output directories
		rm(true, false, &subproj_location!("zerOS").join("bin"));
		rm(true, false, &subproj_location!("zerOS").join("iso-root"));
		mkdir(
			true,
			false,
			&subproj_location!("zerOS")
				.join("bin")
				.join("zerOS-boot-modules")
		);
		mkdir(true, false, &subproj_location!("zerOS").join("iso-root"));

		let mut cmd = std::process::Command::new(cfg.get(&Executable::Cargo));
		cmd.args(&[
			"build",
			format!("--target={json_target}").as_str(),
			"-Z",
			"unstable-options",
			"--artifact-dir",
			"./bin",
			format!("--profile={profile}").as_str()
		]);
		let cmd = CmdIn::new(&subproj_location!("zerOS"), cmd);

		todo!()
	}
}

impl Xtask for XtaskBuildableSubproj
{
	fn execute(&self, globals: &XtaskGlobalOptions)
	{
		match self
		{
			Self::Zeros { .. } => self.execute_zerOS(globals),
			_ => todo!()
		}
	}
}
