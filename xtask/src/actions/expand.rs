use clap::Subcommand;
use tokio::process;

use crate::{
	XtaskGlobalOptions,
	actions::{Xtask, configure::subproj_location},
	doc_comments::subdir,
	env,
	tools::{CmdIn, check_opt}
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Subcommand)]
#[clap(rename_all = "lowercase")]
pub(crate) enum XtaskExpandableSubproj
{
	#[doc = subdir!(zerOS)]
	#[clap(name = "zerOS", alias("zeros"), rename_all = "kebab-case")]
	#[clap(about = subdir!(zerOS))]
	Zeros
	{
		/// Args to be passed to `cargo-expand`
		args: Vec<String>
	},

	#[doc = subdir!(unwindtool)]
	#[clap(alias("unwind-tool"))]
	#[clap(about = subdir!(unwindtool))]
	UnwindTool
	{
		/// Args to be passed to `cargo-expand`
		args: Vec<String>
	},

	#[doc = subdir!(macro-utils)]
	#[clap(about = subdir!(macro-utils))]
	MacroUtils
	{
		/// Args to be passed to `cargo-expand`
		args: Vec<String>
	},

	#[doc = subdir!(proc-macro-utils)]
	#[clap(about = subdir!(proc-macro-utils))]
	ProcMacroUtils
	{
		/// Args to be passed to `cargo-expand`
		args: Vec<String>
	},

	#[doc = subdir!(generate-target)]
	#[clap(about = subdir!(generate-target))]
	GenerateTarget
	{
		/// Args to be passed to `cargo-expand`
		args: Vec<String>
	}
}

impl Xtask for XtaskExpandableSubproj
{
	async fn execute(&self, _globals: &XtaskGlobalOptions)
	{
		let (path, args) = match self
		{
			Self::Zeros { args, .. } => (subproj_location!("zerOS"), args),
			Self::MacroUtils { args, .. } => (subproj_location!("macro-utils"), args),
			Self::ProcMacroUtils { args, .. } => (subproj_location!("proc-macro-utils"), args),
			Self::UnwindTool { args, .. } => (subproj_location!("unwindtool"), args),
			Self::GenerateTarget { args, .. } => (subproj_location!("generate-target"), args)
		};

		let mut cmd = process::Command::new(check_opt!(
			env::var("CARGO").expect("the CARGO environment variable shall be defined")
		));
		cmd.args(&["expand"]);
		cmd.args(args);
		CmdIn::new(path, cmd).finalize().await
	}
}
