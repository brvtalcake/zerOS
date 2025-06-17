use std::{fs, path::PathBuf};

use anyhow::{Ok, Result, anyhow, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use gimli::{BaseAddresses, EhFrame, UnwindContext, UnwindSection};
use object::{
	Endian,
	Endianness,
	ObjectSection,
	ObjectSegment,
	ObjectSymbol,
	ReadRef,
	elf::STT_FUNC,
	read::{
		Object,
		elf::{ElfFile64, Sym}
	}
};

mod troll;

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
		#[arg(short, long, value_enum)]
		/// The architecture to generate debuginfo for.
		arch:   SupportedArch,
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

#[derive(
	Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Hash, strum::AsRefStr,
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
						dump_dwarf_elf64(&elf64)?;
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

fn dump_dwarf_elf64<'data, R: ReadRef<'data>>(file: &ElfFile64<'data, Endianness, R>)
-> Result<()>
{
	let eh_frame_hdr = file
		.section_by_name(".eh_frame_hdr")
		.ok_or(anyhow!("couldn't find \".eh_frame_hdr\" section"))?;
	let eh_frame = file
		.section_by_name(".eh_frame")
		.ok_or(anyhow!("couldn't find \".eh_frame\" section"))?;
	let text = file
		.section_by_name(".text")
		.ok_or(anyhow!("couldn't find \".text\" section"))?;
	let base_addresses = BaseAddresses::default()
		.set_eh_frame_hdr(eh_frame_hdr.address())
		.set_eh_frame(eh_frame.address())
		.set_text(text.address());
	let uncompressed_eh_frame = eh_frame.uncompressed_data()?;
	let unwind_info = EhFrame::new(
		&uncompressed_eh_frame,
		if file.endian().is_little_endian()
		{
			gimli::RunTimeEndian::Little
		}
		else
		{
			gimli::RunTimeEndian::Big
		}
	);
	let mut ctx = UnwindContext::new();
	let strtable = file.elf_symbol_table().strings();
	for sym in file.symbols()
	{
		if sym.elf_symbol().st_type() == STT_FUNC
		{
			let mangled = str::from_utf8(sym.elf_symbol().name(Endianness::Little, strtable)?)?;
			let demangled = demangle(mangled.to_string());
			let unwind = unwind_info
				.unwind_info_for_address(
					&base_addresses,
					&mut ctx,
					sym.address(),
					EhFrame::cie_from_offset
				)
				.map_err(|err| {
					anyhow!(format!(
						"{} (function: {demangled}, address: {})",
						err,
						sym.address()
					))
				})?;
			let fde = unwind_info
				.fde_for_address(&base_addresses, sym.address(), EhFrame::cie_from_offset)
				.map_err(|err| {
					anyhow!(format!(
						"{} (function: {demangled}, address: {})",
						err,
						sym.address()
					))
				})?;
			#[rustfmt::skip]
			println!(
				concat!(
					"  {}:\n",
					"    mangled: {}\n",
					"    address: {:#x}\n",
					"    size: {}\n",
					"    unwind info:\n",
					"      start: {:#x}\n",
					"      size: {}\n",
					"      fde:\n",
					"        start: {:#x}\n",
					"        end: {:#x}\n",
					"        size: {}\n"
				),
				demangled,
				mangled,
				sym.address(),
				sym.size(),
				unwind.start_address(),
				unwind.end_address() - unwind.start_address(),
				fde.initial_address(),
				fde.end_address(),
				fde.end_address().wrapping_sub(fde.initial_address())
			);
		}
	}
	println!("total .eh_frame size: {}", eh_frame.size());
	Ok(())
}
