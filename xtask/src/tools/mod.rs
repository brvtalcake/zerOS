use std::{path::PathBuf, process::abort};

use camino::{Utf8Path, Utf8PathBuf};

pub(crate) mod gentarget;

pub(crate) struct PushDir
{
	initial: PathBuf,
	dir:     Utf8PathBuf
}

impl PushDir
{
	pub(crate) fn new(path: &impl AsRef<Utf8Path>) -> Self
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
	dir:            PushDir,
	pub(crate) cmd: std::process::Command
}

impl CmdIn
{
	#[inline]
	pub(crate) fn new(path: &impl AsRef<Utf8Path>, cmd: std::process::Command) -> Self
	{
		Self {
			dir: PushDir::new(path),
			cmd
		}
	}

	#[inline]
	pub(crate) fn command_directory(&self) -> &Utf8Path
	{
		self.dir.dir.as_path()
	}
}
