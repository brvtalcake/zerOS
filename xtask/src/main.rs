#![feature(variant_count)]
#![feature(decl_macro)]
#![feature(cfg_version)]
#![feature(exit_status_error)]
#![feature(new_range_api)]
#![feature(more_qualified_paths)]
#![feature(sync_unsafe_cell)]
#![feature(phantom_variance_markers)]
#![feature(derive_const)]
#![feature(const_clone)]
#![feature(f128)]
#![feature(specialization)]
#![feature(trait_alias)]
#![feature(slice_pattern)]
#![feature(panic_backtrace_config)]
#![feature(impl_trait_in_bindings)]
#![cfg_attr(test, feature(test))]
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
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};

mod actions;
mod doc_comments;
mod env;
mod limine;
mod requests;
mod term;
mod tools;

use crate::{
	actions::{
		Xtask,
		build::XtaskBuildableSubproj,
		clean::XtaskCleanableSubproj,
		clippy::XtaskClippyableSubproj,
		configure::{XtaskConfigurableSubproj, config_location, init_default_executable_names},
		expand::XtaskExpandableSubproj,
		format::XtaskFormattableSubproj,
		run::XtaskRunnableSubproj
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
	/// Debugging mode.
	///
	/// Only for developers
	debug: bool,

	#[arg(short, long, default_value_t = 0, action = ArgAction::Count)]
	/// Increase verbosity
	verbose: u8,

	#[arg(short, long, default_value_t = 0, action = ArgAction::Count)]
	/// Decrease verbosity
	quiet: u8
}

impl XtaskGlobalOptions
{
	pub(crate) fn verbosity(&self) -> u8
	{
		self.verbose.saturating_sub(self.quiet)
	}

	pub(crate) fn to_verbose_flags(&self) -> Vec<&'static str>
	{
		let verbosity = self.verbosity();
		let mut ret = vec![];
		(0..verbosity).for_each(|_| ret.push("--verbose"));
		ret
	}
}

#[remain::sorted]
#[derive(Debug, Clone, Subcommand)]
#[clap(rename_all = "kebab-case")]
enum XtaskSubcmd
{
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
	/// Configure a subproject
	Configure
	{
		#[command(subcommand)]
		subproj: XtaskConfigurableSubproj
	},
	/// Expand macros in code from a subproject
	Expand
	{
		#[command(subcommand)]
		subproj: XtaskExpandableSubproj
	},
	#[command(visible_alias("fmt"))]
	/// Format code in a subproject
	Format
	{
		#[command(subcommand)]
		subproj: XtaskFormattableSubproj
	},
	Run
	{
		#[command(subcommand)]
		subproj: XtaskRunnableSubproj
	} // TODO: `Test` variant
}

#[remain::sorted]
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
	#[value(alias("arm64"))]
	AArch64,
	#[default]
	#[value(alias("x86-64"), alias("x86_64"))]
	Amd64,
	#[value(alias("arm"))]
	Arm32,
	#[value(alias("avr"))]
	Avr32,
	LoongArch64,
	Mips32,
	Mips64,
	#[value(alias("ppc32"))]
	PowerPC32,
	#[value(alias("ppc64"), alias("ppc"))]
	PowerPC64,
	Riscv32,
	Riscv64,
	Sparc32,
	Sparc64,
	#[value(alias("i386"), alias("i486"), alias("i586"), alias("i686"))]
	X86,
	#[value(alias("s390x"))]
	ZArch
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Endianness
{
	Little,
	Big
}

impl Endianness
{
	pub(crate) const fn qemu_default_for(arch: SupportedArch) -> Option<Self>
	{
		match arch
		{
			SupportedArch::AArch64 => None,
			SupportedArch::Amd64 => None,
			SupportedArch::Arm32 => None,
			SupportedArch::Avr32 => None,
			SupportedArch::LoongArch64 => None,
			SupportedArch::Mips32 => Some(Self::Big),
			SupportedArch::Mips64 => Some(Self::Big),
			SupportedArch::PowerPC32 => None,
			SupportedArch::PowerPC64 => None,
			SupportedArch::Riscv32 => None,
			SupportedArch::Riscv64 => None,
			SupportedArch::Sparc32 => None,
			SupportedArch::Sparc64 => None,
			SupportedArch::X86 => None,
			SupportedArch::ZArch => None
		}
	}
}

#[remain::check]
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
		let global_git_instance = Octocrab::builder().build()?;
		octocrab::initialise(global_git_instance);
		init_default_executable_names();
		colog::init();

		mkdir(false, false, &config_location!()).await;

		let cli = XtaskCLI::parse();
		if cli.globals.debug
		{
			dbg!(&cli);
			std::panic::set_backtrace_style(std::panic::BacktraceStyle::Full);
		}
		else
		{
			std::panic::set_backtrace_style(std::panic::BacktraceStyle::Short);
		}

		#[sorted]
		match &cli.task
		{
			XtaskSubcmd::Build { subproj } => subproj.execute(&cli.globals).await,
			XtaskSubcmd::Clean { subproj } => subproj.execute(&cli.globals).await,
			XtaskSubcmd::Clippy { subproj } => subproj.execute(&cli.globals).await,
			XtaskSubcmd::Configure { subproj } => subproj.execute(&cli.globals).await,
			XtaskSubcmd::Expand { subproj } => subproj.execute(&cli.globals).await,
			XtaskSubcmd::Format { subproj } => subproj.execute(&cli.globals).await,
			XtaskSubcmd::Run { subproj } => subproj.execute(&cli.globals).await
		}

		Ok(())
	})
}

pub(crate) trait IntoArray<T, const N: usize>
{
	fn into_array(self) -> [T; N];
}

impl<T> IntoArray<T, 0> for ()
{
	fn into_array(self) -> [T; 0]
	{
		[]
	}
}

impl<T> IntoArray<T, 1> for (T,)
{
	fn into_array(self) -> [T; 1]
	{
		[self.0]
	}
}

impl<T> IntoArray<T, 2> for (T, T)
{
	fn into_array(self) -> [T; 2]
	{
		[self.0, self.1]
	}
}

impl<T> IntoArray<T, 3> for (T, T, T)
{
	fn into_array(self) -> [T; 3]
	{
		[self.0, self.1, self.2]
	}
}

impl<T> IntoArray<T, 4> for (T, T, T, T)
{
	fn into_array(self) -> [T; 4]
	{
		[self.0, self.1, self.2, self.3]
	}
}

impl<T> IntoArray<T, 5> for (T, T, T, T, T)
{
	fn into_array(self) -> [T; 5]
	{
		[self.0, self.1, self.2, self.3, self.4]
	}
}
