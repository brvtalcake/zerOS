use clap::Subcommand;

use crate::{XtaskGlobalOptions, actions::Xtask, doc_comments::subdir};

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

	#[doc = subdir!(docs)]
	#[clap(about = subdir!(docs))]
	Docs
}

impl Xtask for XtaskClippyableSubproj
{
	async fn execute(&self, globals: &XtaskGlobalOptions)
	{
		todo!()
	}
}
