/// ! TODO: this whole module will only work on Linux distro that stick to
/// freedesktop specs
use std::{
	collections::VecDeque,
	convert::Infallible,
	path::{Path, PathBuf},
	process::ExitStatus,
	str::FromStr,
	sync::LazyLock
};

use anyhow::{Result, anyhow};
use camino::Utf8Path;
use freedesktop_desktop_entry::{DesktopEntry, unicase};
use itertools::Itertools;
use tokio::process;

use crate::{
	env,
	tools::{CmdIn, check_opt}
};

#[derive_const(Clone)]
#[derive(Copy)]
enum DesktopEnv
{
	Unknown,
	Kde,
	Gnome,
	Xfce
}

impl DesktopEnv
{
	fn to_path_buf(&self, base: &Path) -> PathBuf
	{
		match self
		{
			Self::Kde => base.join("KDE-xdg-terminals.list"),

			Self::Gnome => todo!(),
			Self::Xfce => todo!(),

			Self::Unknown => panic!()
		}
	}
}

impl FromStr for DesktopEnv
{
	type Err = Infallible;

	fn from_str(s: &str) -> Result<Self, Self::Err>
	{
		Ok(match s.to_lowercase().as_str()
		{
			"kde" | "plasma" => Self::Kde,
			"gnome" => Self::Gnome,
			"xfce" => Self::Xfce,
			_ => Self::Unknown
		})
	}
}

enum TermListIterState
{
	Start,
	Standard(VecDeque<String>),
	DeSpecific(DesktopEnv, VecDeque<String>),
	Predefined(usize, VecDeque<String>)
}

struct TermListIter
{
	cfgdir: PathBuf,
	state:  TermListIterState
}

fn get_de() -> DesktopEnv
{
	env::var("XDG_CURRENT_DESKTOP")
		.map(|s| DesktopEnv::from_str(&s).unwrap())
		.unwrap_or(DesktopEnv::Unknown)
}

impl Iterator for TermListIter
{
	type Item = String;

	fn next(&mut self) -> Option<Self::Item>
	{
		match self.state
		{
			TermListIterState::Start =>
			{
				self.state = TermListIterState::Standard(
					std::fs::read_to_string(self.cfgdir.join("xdg-terminals.list"))
						.map(|s| s.lines().map(ToOwned::to_owned).collect_vec().into())
						.unwrap_or(VecDeque::new())
				);
				self.next()
			},
			TermListIterState::Standard(ref mut lines) =>
			{
				if !lines.is_empty()
				{
					Some(lines.pop_front().unwrap().trim().to_owned())
				}
				else
				{
					let de = get_de();
					self.state = TermListIterState::DeSpecific(
						de,
						matches!(de, DesktopEnv::Unknown)
							.then_some(VecDeque::new())
							.unwrap_or_else(|| {
								std::fs::read_to_string(de.to_path_buf(&self.cfgdir))
									.map(|s| s.lines().map(ToOwned::to_owned).collect_vec().into())
									.unwrap_or(VecDeque::new())
							})
					);
					self.next()
				}
			},
			TermListIterState::DeSpecific(_, ref mut lines) =>
			{
				if !lines.is_empty()
				{
					Some(lines.pop_front().unwrap().trim().to_owned())
				}
				else
				{
					self.state = TermListIterState::Predefined(0, VecDeque::new());
					self.next()
				}
			},
			TermListIterState::Predefined(..) => todo!()
		}
	}
}

pub(crate) fn get_preferred_appids() -> Option<impl Iterator<Item = String>>
{
	dirs::config_local_dir().map(|dir| {
		TermListIter {
			cfgdir: dir,
			state:  TermListIterState::Start
		}
	})
}

static DESKTOP_ENTRIES: LazyLock<Vec<DesktopEntry>> = LazyLock::new(|| {
	freedesktop_desktop_entry::desktop_entries(
		freedesktop_desktop_entry::get_languages_from_env().as_slice()
	)
});

pub(crate) async fn try_exec_specific(
	pwd: impl AsRef<Utf8Path>,
	appid: String
) -> Result<ExitStatus>
{
	let args =
		freedesktop_desktop_entry::find_app_by_id(&DESKTOP_ENTRIES, unicase::Ascii::new(&appid))
			.ok_or(anyhow!(
				"could not find desktop entry for application id \"{appid}\""
			))?
			.parse_exec()?;
	let mut cmd = process::Command::new(&args[0]);
	cmd.args(&args[1..]);
	CmdIn::new(pwd, cmd).finalize_fallible().await
}

pub(crate) async fn try_exec(pwd: impl AsRef<Utf8Path>) -> Option<ExitStatus>
{
	for appid in check_opt!(
		get_preferred_appids().expect("could not find local user configuration directory")
	)
	{
		if let Ok(exstat) = try_exec_specific(&pwd, appid).await
		{
			return Some(exstat);
		}
	}
	None
}
