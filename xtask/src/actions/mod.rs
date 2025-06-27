use crate::XtaskGlobalOptions;

pub(crate) mod build;
pub(crate) mod clean;
pub(crate) mod clippy;
pub(crate) mod configure;
pub(crate) mod expand;
pub(crate) mod format;

pub(crate) trait Xtask
{
	async fn execute(&self, globals: &XtaskGlobalOptions);
}
