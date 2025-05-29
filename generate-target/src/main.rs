#![deny(missing_docs)]

//! This program generates a JSON target specification

use std::{collections::BTreeMap, fs, path::Path};

use anyhow::Result;
use clap::{Args, Parser, ValueEnum};
use json::JsonValue;
use lazy_static::lazy_static;

#[derive(
	Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Hash, strum::AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
#[clap(rename_all = "lower")]
enum Endian
{
	#[default]
	Little,
	Big
}

#[derive(
	Default,
	Copy,
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	ValueEnum,
	Hash,
	strum::AsRefStr,
	strum::Display,
)]
#[strum(serialize_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
enum RelocModel
{
	#[default]
	Static,
	Pic,
	Pie,
	DynamicNoPic,
	Ropi,
	Rwpi,
	RopiRwpi
}

#[derive(
	Default,
	Copy,
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	ValueEnum,
	Hash,
	strum::AsRefStr,
	strum::Display,
)]
#[strum(serialize_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
enum CodeModel
{
	Tiny,
	Small,
	#[default]
	Kernel,
	Medium,
	Large
}

#[derive(
	Default,
	Copy,
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	ValueEnum,
	Hash,
	strum::AsRefStr,
	strum::Display,
)]
#[strum(serialize_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
enum FramePointer
{
	Always,
	NonLeaf,
	#[default]
	#[value(alias("never"))]
	MayOmit
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
	ZArch,
}

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Hash)]
#[clap(rename_all = "kebab-case")]
enum Config
{
	#[value(alias("initialization"))]
	Init,
	#[default]
	Kernel
}

#[derive(Parser)]
#[command(
	author = "Axel PASCON <axelpascon@nullware.dev>",
	about = "Generate rust's « flexible JSON target specification »",
	long_about
)]
#[clap(rename_all = "kebab-case")]
struct GenerateRustTargetCli
{
	#[arg(long, short, default_value_t = false)]
	/// Whether we want debug output to be printed to `stdout`
	debug: bool,

	#[arg(value_enum, default_value_t, long, short)]
	/// The architecure to generate the target spec for
	arch: SupportedArch,

	#[arg(short, long, value_enum, default_value_t)]
	/// The base config to load
	base_config: Config,

	#[arg(long = "override", value_name = "KEYS")]
	/// When a template is supplied, generate the provided keys even if they are
	/// already provided in the template
	///
	/// Overrideable keys must be separated by commas
	replace: Option<String>,

	#[command(flatten)]
	input: TemplatedInput,

	#[command(flatten)]
	arch_opts: ArchitectureOptions,

	/// The output file path
	out_path: String
}

#[derive(Args, Clone)]
#[group(id = "architecture-options", required = false, multiple = true)]
struct ArchitectureOptions
{
	#[arg(long, default_value = "generic")]
	/// The default CPU to compile for
	///
	/// It is passed to `rustc` with `-C target-cpu=${cpu}`
	cpu: String,

	// #[arg(long = "no-frame-pointer", default_value_t = true, action = ArgAction::SetFalse)]
	#[arg(long, value_enum, default_value_t)]
	/// Whether frame pointers can be omitted
	frame_pointer: FramePointer,

	#[arg(long)]
	/// The target endian
	endian: Option<Endian>,

	#[arg(short = 'p', long = "ptr", value_name = "BIT_WIDTH")]
	/// The target pointer width
	pointer_width: Option<usize>,

	#[arg(short = 'i', long = "int", value_name = "BIT_WIDTH")]
	/// The target C `int` width
	int_width: Option<usize>,

	#[arg(long)]
	/// The LLVM data layout for the target
	data_layout: Option<String>,

	#[arg(long)]
	/// The linker to use
	linker: Option<String>,

	#[arg(long, value_name = "FLAVOR")]
	/// The linker flavor to use
	linker_flavor: Option<String>,

	#[arg(long, short)]
	/// The target features to enable or disable
	features: Option<String>,

	#[arg(long, value_enum, default_value_t)]
	/// The relocation model to use
	reloc_model: RelocModel,

	#[arg(long, value_enum, default_value_t)]
	/// The code model to use
	code_model: CodeModel,

	#[arg(long)]
	/// ABI name to use for disambiguation (e.g. `eabihf`)
	abi: Option<String>,

	#[arg(long)]
	/// An LLVM ABI name
	llvm_abiname: Option<String>,

	#[arg(long)]
	/// Whether to use soft or hard floats
	llvm_floatabi: Option<String>,

	#[arg(long)]
	/// Rustc's soft-float or hard-float ABI
	rustc_abi: Option<String>,

	#[arg(long, hide(true))]
	entry_abi: Option<String>
}

#[derive(Args, Clone)]
#[group(id = "templated-input", required = false, multiple = false)]
struct TemplatedInput
{
	#[arg(long, short, value_name("FILE"))]
	/// The possible template file to parse
	///
	/// It must be a JSON file with valid keys. The keys already present in the
	/// template won't be replaced, and only the ones that aren't already
	/// present will be added (unless the `--override` argument is specified)
	template: Option<String>,

	#[arg(long, short = 's', value_name("STRING"))]
	/// The possible template string to parse
	///
	/// It must be a JSON string with valid keys. The keys already present in
	/// the template won't be replaced, and only the ones that aren't already
	/// present will be added (unless the `--override` argument is specified)
	template_str: Option<String>
}

lazy_static! {
	static ref BASE_TEMPLATES: BTreeMap<(SupportedArch, Config), JsonValue> = {
		let mut tmp: BTreeMap<(SupportedArch, Config), JsonValue> = BTreeMap::new();
		tmp.insert(
			(SupportedArch::Amd64, Config::Init),
			json::object! {
				"llvm-target": "x86_64-unknown-none",
				"target-endian": "little",
				"target-pointer-width": "64",
				"target-c-int-width": "32",
				"data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
				"arch": "x86_64",
				"os": "none",
				"env": "",
				"vendor": "unknown",
				"linker": "rust-lld",
				"linker-flavor": "gnu-lld",
				"features": "-mmx,-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2,+soft-float",
				"dynamic-linking": false,
				"executables": true,
				"relocation-model": "static",
				"code-model": "kernel",
				"disable-redzone": true,
				"frame-pointer": "may-omit",
				"exe-suffix": "",
				"has-rpath": false,
				"no-default-libraries": true,
				"position-independent-executables": false,
				"rustc-abi": "x86-softfloat",
				"cpu": "generic"
			}
		);
		tmp.insert(
			(SupportedArch::Amd64, Config::Kernel),
			json::object! {
				"llvm-target": "x86_64-unknown-none",
				"target-endian": "little",
				"target-pointer-width": "64",
				"target-c-int-width": "32",
				"data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
				"arch": "x86_64",
				"os": "none",
				"env": "",
				"vendor": "unknown",
				"linker": "rust-lld",
				"linker-flavor": "gnu-lld",
				"dynamic-linking": false,
				"executables": true,
				"relocation-model": "static",
				"code-model": "kernel",
				"disable-redzone": true,
				"frame-pointer": "may-omit",
				"exe-suffix": "",
				"has-rpath": false,
				"no-default-libraries": true,
				"position-independent-executables": false,
				"cpu": "generic"
			}
		);
		tmp
	};
}

enum ResultTargetSpecJson
{
	FromTemplate(Vec<String>, bool),
	FromBase
}

struct ResultTargetSpec
{
	json:        JsonValue,
	json_source: ResultTargetSpecJson
}

/// TODO: add a custom help template and custom usage string (?)
fn main() -> Result<()>
{
	let cli = GenerateRustTargetCli::parse();

	let overrideable = if let Some(ref replaceable) = cli.replace
	{
		Some(replaceable.split(',').map(|s| s.to_string()))
	}
	else
	{
		None
	};

	let (json, source) = if let Some(tpath) = cli.input.template
	{
		let overrideable_vec = overrideable
			.map(|splitted| splitted.collect())
			.unwrap_or_default();
		let can_override_any_key = find_ascii_case_insensitive(&overrideable_vec, "all")
			.or(find_ascii_case_insensitive(&overrideable_vec, "any"))
			.or(find_ascii_case_insensitive(&overrideable_vec, "anything"))
			.or(find_ascii_case_insensitive(&overrideable_vec, "everything"))
			.is_some();
		(
			json::parse(&fs::read_to_string(tpath)?)?,
			ResultTargetSpecJson::FromTemplate(overrideable_vec, can_override_any_key)
		)
	}
	else if let Some(tstr) = cli.input.template_str
	{
		let overrideable_vec = overrideable
			.map(|splitted| splitted.collect())
			.unwrap_or_default();
		let can_override_any_key = find_ascii_case_insensitive(&overrideable_vec, "all")
			.or(find_ascii_case_insensitive(&overrideable_vec, "any"))
			.or(find_ascii_case_insensitive(&overrideable_vec, "anything"))
			.or(find_ascii_case_insensitive(&overrideable_vec, "everything"))
			.is_some();
		(
			json::parse(&tstr)?,
			ResultTargetSpecJson::FromTemplate(overrideable_vec, can_override_any_key)
		)
	}
	else
	{
		(
			match cli.arch
			{
				SupportedArch::Amd64 =>
				{
					BASE_TEMPLATES[&(SupportedArch::Amd64, cli.base_config)].to_owned()
				},
				other => todo!("arch {} support is not implemented for now", other.as_ref())
			},
			ResultTargetSpecJson::FromBase
		)
	};

	let mut target_spec = ResultTargetSpec {
		json,
		json_source: source
	};

	let modif_count = match cli.arch
	{
		SupportedArch::Amd64 => target_spec.generate_amd64(&cli.arch_opts),
		other => todo!("arch {} support is not implemented for now", other.as_ref())
	};

	if cli.debug
	{
		println!("{pretty}", pretty = target_spec.json.pretty(4));
		println!("{modif_count} modifications from base template");
	}

	target_spec
		.write_to_file(&cli.out_path)
		.inspect_err(|err| eprintln!("failed to write to {}: {err}", &cli.out_path))
}

impl ResultTargetSpec
{
	fn maybe_update(&mut self, key: &str, value: &JsonValue) -> bool
	{
		match self.json_source
		{
			ResultTargetSpecJson::FromBase =>
			{
				let previous = &self.json[key];
				let ret = *previous != *value;
				self.json[key] = value.to_owned();
				ret
			},
			ResultTargetSpecJson::FromTemplate(ref replaceables, ref can_modify_all) =>
			{
				if *can_modify_all || replaceables.contains(&key.to_owned())
				{
					let previous = &self.json[key];
					let ret = *previous != *value;
					self.json[key] = value.to_owned();
					ret
				}
				else
				{
					false
				}
			},
		}
	}

	fn generate_amd64(&mut self, opts: &ArchitectureOptions) -> usize
	{
		let mut modif_count = 0;
		modif_count += self.maybe_update("cpu", &opts.cpu.to_owned().into()) as usize;
		modif_count += self.maybe_update(
			"frame-pointer",
			&opts.frame_pointer.to_owned().as_ref().into()
		) as usize;
		if let Some(ref endian) = opts.endian
		{
			modif_count +=
				self.maybe_update("target-endian", &endian.to_owned().as_ref().into()) as usize;
		}
		if let Some(ptr_width) = opts.pointer_width
		{
			modif_count += self.maybe_update("target-pointer-width", &ptr_width.to_string().into()) as usize;
		}
		if let Some(int_width) = opts.int_width
		{
			modif_count += self.maybe_update("target-c-int-width", &int_width.to_string().into()) as usize;
		}
		if let Some(ref data_layout) = opts.data_layout
		{
			modif_count += self.maybe_update("data-layout", &data_layout.as_str().into()) as usize;
		}
		if let Some(ref linker) = opts.linker
		{
			modif_count += self.maybe_update("linker", &linker.as_str().into()) as usize;
		}
		if let Some(ref linker_flavor) = opts.linker_flavor
		{
			modif_count +=
				self.maybe_update("linker-flavor", &linker_flavor.as_str().into()) as usize;
		}
		if let Some(ref features) = opts.features
		{
			modif_count += self.maybe_update("features", &features.as_str().into()) as usize;
		}
		modif_count +=
			self.maybe_update("relocation-model", &opts.reloc_model.as_ref().into()) as usize;
		modif_count += self.maybe_update("code-model", &opts.code_model.as_ref().into()) as usize;
		if let Some(ref abi) = opts.abi
		{
			modif_count += self.maybe_update("abi", &abi.as_str().into()) as usize;
		}
		if let Some(ref llvm_abiname) = opts.llvm_abiname
		{
			modif_count +=
				self.maybe_update("llvm-abiname", &llvm_abiname.as_str().into()) as usize;
		}
		if let Some(ref llvm_floatabi) = opts.llvm_floatabi
		{
			modif_count +=
				self.maybe_update("llvm-floatabi", &llvm_floatabi.as_str().into()) as usize;
		}
		if let Some(ref rustc_abi) = opts.rustc_abi
		{
			modif_count += self.maybe_update("rustc-abi", &rustc_abi.as_str().into()) as usize;
		}
		if let Some(ref entry_abi) = opts.entry_abi
		{
			modif_count += self.maybe_update("entry-abi", &entry_abi.as_str().into()) as usize;
		}
		modif_count
	}

	fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()>
	{
		Ok(fs::write(path, self.json.pretty(4))?)
	}
}

fn find_ascii_case_insensitive<H: IntoIterator + ToOwned<Owned = H>, N: ?Sized + AsRef<str>>(
	haystack: &H,
	needle: &N
) -> Option<usize>
where
	<H as IntoIterator>::Item: AsRef<str>
{
	haystack
		.to_owned()
		.into_iter()
		.enumerate()
		.find(|el| el.1.as_ref().eq_ignore_ascii_case(needle.as_ref()))
		.map(|tp| tp.0)
}
