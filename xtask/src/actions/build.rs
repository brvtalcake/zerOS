use clap::{Subcommand, ValueEnum};

use crate::{
	SupportedArch,
	XtaskGlobalOptions,
	actions::{Xtask, configure::ZerosConfig},
	doc_comments::subdir
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum, Default)]
#[clap(rename_all = "kebab-case")]
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
		let cfg = ZerosConfig::load_or_error();
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
