use std::ffi::OsStr;

use itertools::Itertools;
use tokio::process;

use crate::tools::check;

pub(crate) async fn run_with_env<E, EK, EV>(
	objcopy: impl AsRef<OsStr>,
	args: &[impl AsRef<OsStr>],
	env: E
) where
	E: IntoIterator<Item = (EK, EV)>,
	EK: AsRef<OsStr>,
	EV: AsRef<OsStr>
{
	let cmdstr = format!(
		"{} {}",
		objcopy.as_ref().display(),
		args.iter()
			.map(|it| it.as_ref().to_string_lossy())
			.join(" ")
	);
	check!(
		process::Command::new(&objcopy)
			.args(args)
			.envs(env)
			.status()
			.await
			.expect(format!("could not run `{cmdstr}`").as_str())
			.exit_ok()
			.expect(format!("command `{cmdstr}` exited abnormally").as_str())
	)
}

pub(crate) async fn run(objcopy: impl AsRef<OsStr>, args: &[impl AsRef<OsStr>])
{
	run_with_env(objcopy, args, <[(&str, &str); 0]>::default()).await
}
