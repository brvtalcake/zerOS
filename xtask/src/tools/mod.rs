use std::{ffi::OsStr, fmt::Display, io, os::unix::ffi::OsStrExt, path::PathBuf, process::abort};

use anyhow::{Result, anyhow, bail};
use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use log::info;
use tokio::{fs, process, task};

use crate::env;

pub(crate) mod gentarget;
pub(crate) mod mk_iso;
pub(crate) mod objcopy;
pub(crate) mod strip;

pub(crate) struct CmdIn
{
	cmd: process::Command
}

impl CmdIn
{
	#[inline]
	pub(crate) fn new(path: impl AsRef<Utf8Path>, mut cmd: process::Command) -> Self
	{
		cmd.kill_on_drop(true)
			.current_dir(path.as_ref())
			.envs(env::vars_os());
		Self { cmd }
	}

	pub(crate) async fn finalize(mut self)
	{
		info!(
			"running `{executable} {args}`",
			executable = self.cmd.as_std().get_program().display(),
			args = self
				.cmd
				.as_std()
				.get_args()
				.collect_vec()
				.join(OsStr::from_bytes(b" "))
				.display()
		);
		check!(
			self.cmd
				.status()
				.await
				.expect("failed to spawn process")
				.exit_ok()
				.expect("program terminated abnormally")
		)
	}
}

pub(crate) async fn rm_fallible(recursive: bool, strict: bool, path: &Utf8Path) -> Result<()>
{
	if path.is_dir()
	{
		if recursive
		{
			info!("removing directory {path} and its content");
			Ok(fs::remove_dir_all(path).await?)
		}
		else
		{
			info!("removing directory {path}");
			Ok(fs::remove_dir(path).await?)
		}
	}
	else if path.is_file() || path.is_symlink()
	{
		info!(
			"{} {path}",
			path.is_file().then_some("removing").unwrap_or("unlinking")
		);
		Ok(fs::remove_file(path).await?)
	}
	else if strict
	{
		Err(anyhow!("\"{path}\" does not exist or can not be removed"))
	}
	else
	{
		Ok(())
	}
}

pub(crate) async fn mkdir_fallible(parents: bool, strict: bool, path: &Utf8Path) -> Result<()>
{
	let else_fn = |err: io::Error| {
		if !strict && err.kind() == io::ErrorKind::AlreadyExists && path.is_dir()
		{
			Ok(())
		}
		else
		{
			Err(err)
		}
	};
	if parents
	{
		info!("creating directory {path} and its parents");
		Ok(fs::create_dir_all(path).await.or_else(&else_fn)?)
	}
	else
	{
		info!("creating directory {path}");
		Ok(fs::create_dir(path).await.or_else(&else_fn)?)
	}
}

pub(crate) async fn cp_fallible(from: &Utf8Path, to: &Utf8Path) -> Result<()>
{
	let is_dir = from.is_dir();

	info!(
		"{prefix}copying {from} to {to}",
		prefix = is_dir.then_some("recursively ").unwrap_or("")
	);

	if is_dir
	{
		mkdir_fallible(false, false, to).await?;

		for maybe_entry in from.read_dir_utf8()?
		{
			let entry = maybe_entry?;

			let source = entry.path();
			let dest = to.join(entry.file_name());

			Box::pin(cp_fallible(source, &dest)).await?;
		}

		Ok(())
	}
	else
	{
		Ok(fs::copy(from, to).await.map(|_| ())?)
	}
}

pub(crate) async fn rm(recursive: bool, strict: bool, path: &Utf8Path)
{
	check!(
		rm_fallible(recursive, strict, path)
			.await
			.expect("could not delete directory")
	)
}

pub(crate) async fn mkdir(parents: bool, strict: bool, path: &Utf8Path)
{
	check!(
		mkdir_fallible(parents, strict, path)
			.await
			.expect("could not create directory")
	)
}

pub(crate) async fn cp(from: &Utf8Path, to: &Utf8Path)
{
	check!(cp_fallible(from, to).await.expect("could not copy file"))
}

pub(crate) macro check_handle($handle:expr, $msg:expr) {
	$crate::tools::check!($handle.await.expect($msg))
}

pub(crate) macro check
{
	(@munch($($munched:tt)*) .expect($string:expr $(=> $hint:expr $(, $cont:expr)*)?) $($others:tt)*) => {
		$crate::tools::check!(
			@munch(
				($($munched)*).unwrap_or_else(
					|err| {
						::log::error!(
							"{___err_report}",
							___err_report = (err).report_error_with_prefix($string)
						);
						$(
							::log::error!(
								"hint: {___hint}",
								___hint = $hint
							);
							$(
								::log::error!(
									"      {___cont}",
									___cont = $cont.replace("\n", "\n      ")
								);
							)*
						)?
						::std::process::abort();
					}
				)
			)
			$($others)*
		)
	},
	(@munch($($munched:tt)*) $first:tt $($others:tt)*) => {
		$crate::tools::check!(@munch($($munched)* $first) $($others)*)
	},
	(@munch($($munched:tt)*)) => {
		$($munched)*
	},
	($($tokens:tt)*) => {
		$crate::tools::check!(@munch() $($tokens)*)
	},
}

pub(crate) macro check_opt
{
	(@munch($($munched:tt)*) .expect($string:expr $(=> $hint:expr $(, $cont:expr)*)?) $($others:tt)*) => {
		$crate::tools::check_opt!(
			@munch(
				($($munched)*).unwrap_or_else(
					|| {
						::log::error!(
							"{___prefix}",
							___prefix = $string
						);
						$(
							::log::error!(
								"hint: {___hint}",
								___hint = $hint
							);
							$(
								::log::error!(
									"      {___cont}",
									___cont = $cont.replace("\n", "\n      ")
								);
							)*
						)?
						::std::process::abort();
					}
				)
			)
			$($others)*
		)
	},
	(@munch($($munched:tt)*) $first:tt $($others:tt)*) => {
		$crate::tools::check_opt!(@munch($($munched)* $first) $($others)*)
	},
	(@munch($($munched:tt)*)) => {
		$($munched)*
	},
	($($tokens:tt)*) => {
		$crate::tools::check_opt!(@munch() $($tokens)*)
	},
}

pub(crate) trait ReportError: Display
{
	fn report_error(&self) -> String;
	fn report_error_with_prefix(&self, prefix: impl Display) -> String;
}

impl<T: Display> ReportError for T
{
	default fn report_error(&self) -> String
	{
		format!("{self}")
	}

	default fn report_error_with_prefix(&self, prefix: impl Display) -> String
	{
		format!("{prefix}: {}", self.report_error())
	}
}

impl<T: Display> ReportError for T
where
	T: std::error::Error
{
	fn report_error(&self) -> String
	{
		self::report_error(self as &dyn std::error::Error, Option::<String>::None)
	}

	fn report_error_with_prefix(&self, prefix: impl Display) -> String
	{
		self::report_error(self as &dyn std::error::Error, Some(prefix))
	}
}

fn report_error(mut err: &(dyn std::error::Error), prefix: Option<impl Display>) -> String
{
	if let Some(prefix) = prefix
	{
		let mut i = 0;
		let mut s = format!("{prefix}:\n\t{i}: {err}");
		i += 1;
		while let Some(src) = err.source()
		{
			s.push_str(&format!("\n\t{i}: {}", src));
			err = src;
			i += 1;
		}
		s
	}
	else
	{
		let mut i = 0;
		let mut s = format!("{i}: {err}");
		i += 1;
		while let Some(src) = err.source()
		{
			s.push_str(&format!("\n{i}: {}", src));
			err = src;
			i += 1;
		}
		s
	}
}
