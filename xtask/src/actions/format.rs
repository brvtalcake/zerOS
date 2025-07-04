use std::str::FromStr;

use camino::Utf8PathBuf;
use clap::{ArgAction, Subcommand};
use tokio::process;

use crate::{
	XtaskGlobalOptions,
	actions::{
		Xtask,
		configure::{get_topdir, subproj_location}
	},
	doc_comments::subdir,
	env,
	tools::{CmdIn, check, check_opt}
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Subcommand)]
#[clap(rename_all = "lowercase")]
pub(crate) enum XtaskFormattableSubproj
{
	#[doc = subdir!(zerOS)]
	#[clap(name = "zerOS", alias("zeros"), rename_all = "kebab-case")]
	#[clap(about = subdir!(zerOS))]
	Zeros
	{
		#[arg(short, long, default_value_t = false, action = ArgAction::SetTrue)]
		/// Only check if subproject is formatted
		check: bool,

		#[arg(short = 'p', long)]
		/// Provide an alternative config file for `cargo fmt`
		config: Option<String>
	},

	#[doc = subdir!(unwindtool)]
	#[clap(alias("unwind-tool"))]
	#[clap(about = subdir!(unwindtool))]
	UnwindTool
	{
		#[arg(short, long, default_value_t = false, action = ArgAction::SetTrue)]
		/// Only check if subproject is formatted
		check: bool,

		#[arg(short = 'p', long)]
		/// Provide an alternative config file for `cargo fmt`
		config: Option<String>
	},

	#[doc = subdir!(macro-utils)]
	#[clap(about = subdir!(macro-utils))]
	MacroUtils
	{
		#[arg(short, long, default_value_t = false, action = ArgAction::SetTrue)]
		/// Only check if subproject is formatted
		check: bool,

		#[arg(short = 'p', long)]
		/// Provide an alternative config file for `cargo fmt`
		config: Option<String>
	},

	#[doc = subdir!(proc-macro-utils)]
	#[clap(about = subdir!(proc-macro-utils))]
	ProcMacroUtils
	{
		#[arg(short, long, default_value_t = false, action = ArgAction::SetTrue)]
		/// Only check if subproject is formatted
		check: bool,

		#[arg(short = 'p', long)]
		/// Provide an alternative config file for `cargo fmt`
		config: Option<String>
	},

	#[doc = subdir!(docs)]
	#[clap(about = subdir!(docs))]
	Docs
	{
		#[arg(short, long, default_value_t = false, action = ArgAction::SetTrue)]
		/// Only check if subproject is formatted
		check: bool,

		#[arg(short = 'p', long)]
		/// Provide an alternative config file for LaTeX formatter
		config: Option<String>
	},

	#[doc = subdir!(generate-target)]
	#[clap(about = subdir!(generate-target))]
	GenerateTarget
	{
		#[arg(short, long, default_value_t = false, action = ArgAction::SetTrue)]
		/// Only check if subproject is formatted
		check: bool,

		#[arg(short = 'p', long)]
		/// Provide an alternative config file for `cargo fmt`
		config: Option<String>
	}
}

impl Xtask for XtaskFormattableSubproj
{
	async fn execute(&self, globals: &XtaskGlobalOptions)
	{
		let verbose_flags = globals.to_verbose_flags();
		let (path, check_mode, cfg) = match self
		{
			Self::Zeros { check, config } =>
			{
				(
					subproj_location!("zerOS"),
					*check,
					config
						.clone()
						.map(|s| {
							check!(
								Utf8PathBuf::from_str(&s)
									.expect("invalid `cargo fmt` config file path")
							)
						})
						.unwrap_or_else(|| get_topdir().into())
				)
			},
			Self::MacroUtils { check, config } =>
			{
				(
					subproj_location!("macro-utils"),
					*check,
					config
						.clone()
						.map(|s| {
							check!(
								Utf8PathBuf::from_str(&s)
									.expect("invalid `cargo fmt` config file path")
							)
						})
						.unwrap_or_else(|| get_topdir().into())
				)
			},
			Self::ProcMacroUtils { check, config } =>
			{
				(
					subproj_location!("proc-macro-utils"),
					*check,
					config
						.clone()
						.map(|s| {
							check!(
								Utf8PathBuf::from_str(&s)
									.expect("invalid `cargo fmt` config file path")
							)
						})
						.unwrap_or_else(|| get_topdir().into())
				)
			},
			Self::UnwindTool { check, config } =>
			{
				(
					subproj_location!("unwindtool"),
					*check,
					config
						.clone()
						.map(|s| {
							check!(
								Utf8PathBuf::from_str(&s)
									.expect("invalid `cargo fmt` config file path")
							)
						})
						.unwrap_or_else(|| get_topdir().into())
				)
			},
			Self::Docs { check, config } =>
			{
				(
					subproj_location!("docs"),
					*check,
					config
						.clone()
						.map(|s| {
							check!(
								Utf8PathBuf::from_str(&s)
									.expect("invalid `cargo fmt` config file path")
							)
						})
						.unwrap_or_else(|| get_topdir().into())
				)
			},
			Self::GenerateTarget { check, config } =>
			{
				(
					subproj_location!("generate-target"),
					*check,
					config
						.clone()
						.map(|s| {
							check!(
								Utf8PathBuf::from_str(&s)
									.expect("invalid `cargo fmt` config file path")
							)
						})
						.unwrap_or_else(|| get_topdir().into())
				)
			},
		};

		if !matches!(self, Self::Docs { .. })
		{
			let mut cmd = process::Command::new(check_opt!(
				env::var("CARGO").expect("the CARGO environment variable shall be defined")
			));
			cmd.arg("fmt");
			cmd.args(verbose_flags);
			if check_mode
			{
				cmd.arg("--check");
			}
			cmd.args(&["--", "--config-path", cfg.as_str()]);
			CmdIn::new(path, cmd).finalize().await
		}
		else
		{
			todo!()
		}
	}
}
