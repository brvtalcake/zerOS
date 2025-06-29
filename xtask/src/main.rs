#![feature(variant_count)]
#![feature(decl_macro)]
#![feature(cfg_version)]
#![feature(exit_status_error)]
#![feature(panic_backtrace_config)]
#![cfg_attr(not(version("1.89")), feature(file_lock))]
#![forbid(unused_must_use)]
#![allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]

#[allow(unused_imports)]
#[macro_use]
extern crate scopeguard;
#[macro_use]
extern crate eager2;

use anyhow::{Ok, Result};
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

mod actions;
mod doc_comments;
mod limine;
mod tools;

use crate::{
	actions::{
		Xtask,
		build::XtaskBuildableSubproj,
		clean::XtaskCleanableSubproj,
		clippy::XtaskClippyableSubproj,
		configure::{XtaskConfigurableSubproj, config_location, init_default_executable_names},
		expand::XtaskExpandableSubproj,
		format::XtaskFormattableSubproj
	},
	tools::{check, mkdir}
};

#[derive(Debug, Clone, Parser)]
#[command(
	author = "Axel PASCON <axelpascon@nullware.dev>",
	about = "CLI tool to drive various other tools to be applied to zerOS",
	long_about
)]
#[clap(rename_all = "kebab-case")]
struct XtaskCLI
{
	#[command(subcommand)]
	task:    XtaskSubcmd,
	#[command(flatten)]
	globals: XtaskGlobalOptions
}

#[derive(Debug, Clone, Args)]
struct XtaskGlobalOptions
{
	#[arg(short, long, default_value_t = false, action = ArgAction::SetTrue)]
	debug: bool
}

#[derive(Debug, Clone, Subcommand)]
#[clap(rename_all = "kebab-case")]
enum XtaskSubcmd
{
	/// Configure a subproject
	Configure
	{
		#[command(subcommand)]
		subproj: XtaskConfigurableSubproj
	},
	/// Build a subproject
	Build
	{
		#[command(subcommand)]
		subproj: XtaskBuildableSubproj
	},
	/// Clean a subproject
	Clean
	{
		#[command(subcommand)]
		subproj: XtaskCleanableSubproj
	},
	/// Run `cargo clippy` on a subproject
	Clippy
	{
		#[command(subcommand)]
		subproj: XtaskClippyableSubproj
	},
	/// Format code in a subproject
	Format
	{
		#[command(subcommand)]
		subproj: XtaskFormattableSubproj
	},
	/// Expand macros in code from a subproject
	Expand
	{
		#[command(subcommand)]
		subproj: XtaskExpandableSubproj
	} // TODO: `Run` variant
}

#[derive(
	Serialize,
	Deserialize,
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	Hash,
	ValueEnum,
	Default,
	strum::AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
#[clap(rename_all = "lower")]
enum SupportedArch
{
	#[default]
	#[value(alias("x86-64"), alias("x86_64"))]
	Amd64,
	#[value(alias("i386"), alias("i486"), alias("i586"), alias("i686"))]
	X86,
	#[value(alias("arm64"))]
	AArch64,
	#[value(alias("arm"))]
	Arm32,
	Riscv32,
	Riscv64,
	#[value(alias("ppc32"))]
	PowerPC32,
	#[value(alias("ppc64"), alias("ppc"))]
	PowerPC64,
	Sparc32,
	Sparc64,
	Mips32,
	Mips64,
	#[value(alias("avr"))]
	Avr32,
	LoongArch64,
	#[value(alias("s390x"))]
	ZArch
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Endianness
{
	Little,
	Big
}

fn main() -> Result<()>
{
	let tokio = check!(
		tokio::runtime::Builder::new_multi_thread()
			.enable_all()
			.worker_threads(num_cpus::get() * 2)
			.thread_name("xtask-tokio-worker")
			.build()
			.expect("failed to create tokio runtime")
	);
	tokio.block_on(async {
		init_default_executable_names();
		colog::init();
		std::panic::set_backtrace_style(std::panic::BacktraceStyle::Short);

		mkdir(false, false, &config_location!()).await;

		let cli = XtaskCLI::parse();
		if cli.globals.debug
		{
			dbg!(&cli);
			unsafe {
				std::env::set_var("RUST_BACKTRACE", "full");
			}
		}
		else
		{
			unsafe {
				std::env::set_var("RUST_BACKTRACE", "1");
			}
		}

		match &cli.task
		{
			XtaskSubcmd::Configure { subproj } => subproj.execute(&cli.globals).await,
			XtaskSubcmd::Build { subproj } => subproj.execute(&cli.globals).await,
			XtaskSubcmd::Clean { subproj } => subproj.execute(&cli.globals).await,
			XtaskSubcmd::Clippy { subproj } => subproj.execute(&cli.globals).await,
			XtaskSubcmd::Format { subproj } => subproj.execute(&cli.globals).await,
			XtaskSubcmd::Expand { subproj } => subproj.execute(&cli.globals).await
		}

		Ok(())
	})
}

pub(crate) trait IntoArray<T, const N: usize>
where
	T: Sized
{
	fn into_array(self) -> [T; N];
}

impl<T: Sized> IntoArray<T, 2> for (T, T)
{
	fn into_array(self) -> [T; 2]
	{
		[self.0, self.1]
	}
}

impl<T: Sized> IntoArray<T, 3> for (T, T, T)
{
	fn into_array(self) -> [T; 3]
	{
		[self.0, self.1, self.2]
	}
}

impl<T: Sized> IntoArray<T, 4> for (T, T, T, T)
{
	fn into_array(self) -> [T; 4]
	{
		[self.0, self.1, self.2, self.3]
	}
}

impl<T: Sized> IntoArray<T, 5> for (T, T, T, T, T)
{
	fn into_array(self) -> [T; 5]
	{
		[self.0, self.1, self.2, self.3, self.4]
	}
}
