use clap::Subcommand;
use tokio::process;

use crate::{
	XtaskGlobalOptions,
	actions::{Xtask, configure::subproj_location},
	doc_comments::subdir,
	env,
	tools::{CmdIn, check_opt}
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Subcommand)]
#[clap(rename_all = "lowercase")]
pub(crate) enum XtaskClippyableSubproj
{
	#[doc = subdir!(zerOS)]
	#[clap(name = "zerOS", alias("zeros"), rename_all = "kebab-case")]
	#[clap(about = subdir!(zerOS))]
	Zeros,

	#[doc = subdir!(unwindtool)]
	#[clap(alias("unwind-tool"))]
	#[clap(about = subdir!(unwindtool))]
	UnwindTool,

	#[doc = subdir!(macro-utils)]
	#[clap(about = subdir!(macro-utils))]
	MacroUtils,

	#[doc = subdir!(proc-macro-utils)]
	#[clap(about = subdir!(proc-macro-utils))]
	ProcMacroUtils,

	#[doc = subdir!(generate-target)]
	#[clap(about = subdir!(generate-target))]
	GenerateTarget
}

impl Xtask for XtaskClippyableSubproj
{
	async fn execute(&self, _globals: &XtaskGlobalOptions)
	{
		let path = match self
		{
			Self::Zeros => subproj_location!("zerOS"),
			Self::MacroUtils => subproj_location!("macro-utils"),
			Self::ProcMacroUtils => subproj_location!("proc-macro-utils"),
			Self::UnwindTool => subproj_location!("unwindtool"),
			Self::GenerateTarget => subproj_location!("generate-target")
		};

		let mut cmd = process::Command::new(check_opt!(
			env::var("CARGO").expect("the CARGO environment variable shall be defined")
		));
		cmd.args(&["clippy"]);
		CmdIn::new(path, cmd).finalize().await
	}
}
