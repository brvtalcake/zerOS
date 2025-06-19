use clap::Subcommand;

use crate::doc_comments::subdir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Subcommand)]
#[clap(rename_all = "lowercase")]
pub(crate) enum XtaskConfigurableSubproj
{
	#[doc = subdir!(zerOS)]
	#[clap(name = "zerOS", alias("zeros"), rename_all = "kebab-case")]
	#[clap(about = subdir!(zerOS))]
	Zeros
}
