use clap::Subcommand;

use crate::{XtaskGlobalOptions, actions::Xtask, doc_comments::subdir};

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
	async fn execute(&self, globals: &XtaskGlobalOptions)
	{
		todo!()
	}
}
