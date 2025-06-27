use std::{ffi::OsStr, fmt::Display};

use itertools::Itertools;
use log::{error, info};
use tokio::process;

use crate::tools::{check, objcopy};

pub(crate) enum StripFlavor<'path>
{
	Unix
	{
		objcopy: &'path dyn AsRef<OsStr>
	},
	ElfUtils
}

async fn do_it(strip: impl AsRef<OsStr>, args: &[impl AsRef<OsStr>])
{
	let cmdstr = format!(
		"{} {}",
		strip.as_ref().display(),
		args.iter()
			.map(|it| it.as_ref().to_string_lossy())
			.join(" ")
	);
	check!(
		process::Command::new(strip)
			.args(args)
			.status()
			.await
			.expect(format!("could not run `{cmdstr}`").as_str())
			.exit_ok()
			.expect(format!("command `{cmdstr}` exited abnormally").as_str())
	);
}

pub(crate) async fn run(
	strip: impl AsRef<OsStr>,
	flavor: StripFlavor<'_>,
	infile: impl AsRef<str> + Display,
	stripped: impl AsRef<str>,
	debuginfo: Option<impl AsRef<str>>
)
{
	info!("striping binary {infile}");
	match (debuginfo, flavor)
	{
		(Some(dbgout), StripFlavor::Unix { objcopy }) =>
		{
			objcopy::run(
				objcopy,
				&["--only-keep-debug", infile.as_ref(), dbgout.as_ref()]
			)
			.await;
			do_it(strip, &["-S", infile.as_ref(), "-o", stripped.as_ref()]).await;
		},
		(None, StripFlavor::Unix { .. }) =>
		{
			do_it(strip, &["-S", infile.as_ref(), "-o", stripped.as_ref()]).await;
		},
		(Some(dbgout), StripFlavor::ElfUtils) =>
		{
			do_it(
				strip,
				&[
					"-f",
					dbgout.as_ref(),
					infile.as_ref(),
					"-o",
					stripped.as_ref()
				]
			)
			.await;
		},
		(None, StripFlavor::ElfUtils) =>
		{
			do_it(strip, &[infile.as_ref(), "-o", stripped.as_ref()]).await;
		},
		_ =>
		{
			error!("unknown strip binary");
			std::process::abort();
		}
	}
}
