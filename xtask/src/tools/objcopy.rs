use std::ffi::OsStr;

use camino::Utf8PathBuf;
use itertools::Itertools;
use tokio::process;

use crate::tools::{CmdIn, check};

pub(crate) async fn run_with_env<E, EK, EV>(
	objcopy: impl AsRef<OsStr>,
	args: &[impl AsRef<OsStr>],
	env: E
) where
	E: IntoIterator<Item = (EK, EV)>,
	EK: AsRef<OsStr>,
	EV: AsRef<OsStr>
{
	let mut cmd = process::Command::new(&objcopy);
	cmd.args(args).envs(env);
	CmdIn::new(
		check!(
			Utf8PathBuf::from_path_buf(check!(
				std::env::current_dir().expect("could not retrieve current working directory")
			))
			.map_err(|p| p.display().to_string())
			.expect("PathBuf contains non UTF-8 characters")
		),
		cmd
	)
	.finalize()
	.await
}

pub(crate) async fn run(objcopy: impl AsRef<OsStr>, args: &[impl AsRef<OsStr>])
{
	run_with_env(objcopy, args, <[(&str, &str); 0]>::default()).await
}
