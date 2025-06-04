use std::{fs, path::PathBuf};

use anyhow::{Ok, Result, anyhow, bail};
use clap::{Args, Parser, Subcommand};
use object::{
	elf::STT_FUNC, read::{
		elf::{ElfFile64, Sym}, Object
	}, Endian, Endianness, ObjectSection, ObjectSegment, ObjectSymbol, ReadRef
};

#[derive(Parser)]
#[command(
	author = "Axel PASCON <axelpascon@nullware.dev>",
	about = "Generate or inspect zerOS' custom unwind tables, a.k.a. « TROLL »",
	long_about = "« TROLL » stands for « Terrible Rewinding ORC-Like Language », and is an \
	              obvious reference to ELF, DWARF and Linux's ORC unwinder."
)]
#[clap(rename_all = "kebab-case")]
struct UnwindToolCli
{
	#[command(subcommand)]
	format: UnwindToolFormat
}

#[derive(Subcommand, strum::AsRefStr)]
#[clap(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
enum UnwindToolAction
{
	#[command(alias("print"))]
	/// Inspect (i.e. print or dump) the related sections from the given
	/// executable
	Inspect
	{
		#[clap(value_name = "FILE")]
		/// The executable to inspect.
		executable: PathBuf
	},

	/// Generate the needed unwinding informations in the selected format (note
	/// that it is a no-op for the DWARF format)
	Generate
	{
		#[clap(value_name = "FROM")]
		/// The input executable to modify.
		input:  PathBuf,
		#[command(flatten)]
		output: UnwindToolGenerationOutput
	}
}

#[derive(Subcommand)]
#[clap(rename_all = "kebab-case")]
enum UnwindToolFormat
{
	/// Operate on the DWARF format.
	Dwarf
	{
		#[command(subcommand)]
		/// The action to effectuate.
		action: UnwindToolAction
	},
	/// Operate on the TROLL format.
	Troll
	{
		#[command(subcommand)]
		/// The action to effectuate.
		action: UnwindToolAction
	}
}

#[derive(Args, Clone)]
#[group(id = "generation-output", required = true, multiple = false)]
#[clap(rename_all = "kebab-case")]
struct UnwindToolGenerationOutput
{
	#[arg(short, long)]
	/// Modify the input file in-place.
	in_place: bool,

	#[arg(short = 'o', long = "output", value_name = "TO")]
	/// The output executable to modify.
	file: Option<PathBuf>
}

fn main() -> Result<()>
{
	let cli = UnwindToolCli::parse();
	match &cli.format
	{
		UnwindToolFormat::Dwarf { action } =>
		{
			if let UnwindToolAction::Inspect { executable } = action
			{
				let content = fs::read(executable)?;
				let parsed = object::File::parse(&*content)?;
				match parsed
				{
					object::File::Elf64(elf64) =>
					{
						dump_elf64(&elf64)?;
					},
					_ => bail!("unsupported format !")
				}
			}
			else
			{
				bail!(format!(
					"the requested action \"{}\" is not supported for DWARF format !",
					action.as_ref()
				));
			}
		},
		UnwindToolFormat::Troll { action } => todo!()
	}
	Ok(())
}

fn demangle(fname: String) -> String
{
	symbolic_demangle::demangle(&fname).to_string()
}

fn dump_elf64<'data, R: ReadRef<'data>>(file: &ElfFile64<'data, Endianness, R>) -> Result<()>
{
	let strtable = file.elf_symbol_table().strings();
	for sym in file.symbols()
	{
		if sym.elf_symbol().st_type() == STT_FUNC
		{
			let mangled = str::from_utf8(sym.elf_symbol().name(Endianness::Little, strtable)?)?;
			#[rustfmt::skip]
			println!(
				concat!(
					"  {}:\n",
					"    mangled: {}\n",
					"    address: 0x{:x}\n",
					"    size   : {}\n",
				),
				demangle(mangled.to_string()),
				mangled,
				sym.address(),
				sym.size()
			);
		}
	}
	Ok(())
}
