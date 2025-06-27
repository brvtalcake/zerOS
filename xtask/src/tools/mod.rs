use std::{ffi::OsStr, io, os::unix::ffi::OsStrExt, path::PathBuf, process::abort};

use anyhow::{Result, anyhow};
use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use log::info;
use tokio::{fs, process};

pub(crate) mod gentarget;
pub(crate) mod mk_iso;
pub(crate) mod objcopy;
pub(crate) mod strip;

pub(crate) struct PushDir
{
	initial: PathBuf,
	dir:     Utf8PathBuf
}

impl PushDir
{
	pub(crate) fn new(path: impl AsRef<Utf8Path>) -> Self
	{
		assert!(path.as_ref().is_dir());
		let this = Self {
			initial: std::env::current_dir()
				.map(|p| p.canonicalize().unwrap())
				.unwrap(),
			dir:     path.as_ref().canonicalize_utf8().unwrap()
		};
		std::env::set_current_dir(this.dir.as_std_path()).unwrap();
		this
	}
}

impl Drop for PushDir
{
	fn drop(&mut self)
	{
		std::env::set_current_dir(&self.initial).unwrap_or_else(|_| abort())
	}
}

pub(crate) struct CmdIn
{
	dir: PushDir,
	cmd: process::Command
}

impl CmdIn
{
	#[inline]
	pub(crate) fn new(path: impl AsRef<Utf8Path>, cmd: process::Command) -> Self
	{
		Self {
			dir: PushDir::new(path),
			cmd
		}
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

pub(crate) macro check
{
	(@munch($($munched:tt)*) .expect($string:expr $(=> $hint:expr $(, $cont:expr)*)?) $($others:tt)*) => {
		$crate::tools::check!(
			@munch(
				($($munched)*).unwrap_or_else(
					|err| {
						::log::error!(
							"{___prefix}: {err}",
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
