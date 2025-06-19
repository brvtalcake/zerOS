#![feature(decl_macro)]

use clap::{Args, Parser, Subcommand, ValueEnum};

mod actions;
mod doc_comments;

use crate::actions::{
	build::XtaskBuildableSubproj,
	clean::XtaskCleanableSubproj,
	clippy::XtaskClippyableSubproj,
	configure::XtaskConfigurableSubproj,
	format::XtaskFormattableSubproj
};

#[derive(Debug, Clone, Parser)]
#[command(
	author = "Axel PASCON <axelpascon@nullware.dev>",
	about = "CLI tool to drive various other tools to be applied to zerOS",
	long_about
)]
#[clap(rename_all = "kebab-case")]
struct Xtask
{
	#[command(subcommand)]
	task: XtaskSubcmd
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
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum, Default)]
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

fn main()
{
	let cli = Xtask::parse();
	dbg!(cli);
}
