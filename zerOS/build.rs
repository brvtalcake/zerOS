#![recursion_limit = "512"]
#![allow(unused_macros)]
#![feature(exit_status_error)]

use core::panic;
use std::{
	ffi::OsString,
	fs,
	io::{self, Write},
	path::{self, Path, PathBuf},
	process::{Command, exit},
	sync::OnceLock
};

use cfg_aliases::cfg_aliases;
use macro_utils::{callback, identity_expand};
use proc_macro_utils::array_size;
use serde::Deserialize;
use strum::VariantNames;

macro_rules! to_cargo {
	($cfgstring:expr => $what:expr) => {
		println!("cargo::{}={}", $cfgstring, $what)
	};
}

macro_rules! from_cargo {
	($cfgvar:expr) => {
		::std::env::var_os($cfgvar)
	};
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct KConfig
{
	boot: KConfigBoot
}

impl Default for KConfig
{
	fn default() -> Self
	{
		Self {
			boot: KConfigBoot::default()
		}
	}
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[serde(rename = "boot")]
struct KConfigBoot
{
	bootloader: KConfigBootBootloader
}

impl Default for KConfigBoot
{
	fn default() -> Self
	{
		Self {
			bootloader: KConfigBootBootloader::Limine
		}
	}
}

#[derive(
	Deserialize,
	Debug,
	Default,
	Clone,
	Copy,
	strum::AsRefStr,
	strum::EnumString,
	strum::VariantNames,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
#[serde(rename = "bootloader")]
enum KConfigBootBootloader
{
	#[default]
	Limine,
	GRUB2,
	UEFI
}

fn parse_kconfig() -> KConfig
{
	toml::from_str(fs::read_to_string("./kconfig.toml").unwrap_or_else(
		|err| {
			to_cargo!("error" => format!("couldn't read kernel configuration file `./kconfig.toml`: {err}"));
			exit(1)
		}
	).as_str()).unwrap_or_else(
		|err| {
			to_cargo!("error" => format!("couldn't parse kernel configuration file `./kconfig.toml`: {err}"));
			exit(1)
		}
	)
}

fn get_outdir() -> Option<&'static String>
{
	static OUTDIR: OnceLock<Option<String>> = OnceLock::new();
	OUTDIR
		.get_or_init(|| from_cargo!("OUT_DIR").map(|val| val.into_string().unwrap()))
		.as_ref()
}

/// returns target triple
fn get_target_arch() -> Option<&'static String>
{
	static TARGET: OnceLock<Option<String>> = OnceLock::new();
	TARGET
		.get_or_init(|| from_cargo!("TARGET").map(|val| val.into_string().unwrap()))
		.as_ref()
}

fn get_opt_lvl() -> Option<&'static isize>
{
	static OPTLVL: OnceLock<Option<isize>> = OnceLock::new();
	OPTLVL
		.get_or_init(|| {
			from_cargo!("OPT_LEVEL").map(|val| val.into_string().unwrap().parse().unwrap())
		})
		.as_ref()
}

fn get_profile() -> Option<&'static String>
{
	static PROFILE: OnceLock<Option<String>> = OnceLock::new();
	PROFILE
		.get_or_init(|| from_cargo!("PROFILE").map(|val| val.into_string().unwrap()))
		.as_ref()
}

fn get_target_cpu() -> Option<&'static String>
{
	static TARGET_CPU: OnceLock<Option<String>> = OnceLock::new();
	TARGET_CPU
		.get_or_init(|| from_cargo!("ZEROS_TARGET_CPU").map(|val| val.into_string().unwrap()))
		.as_ref()
}

fn get_target_ptr_width() -> Option<&'static usize>
{
	static TARGET_POINTER_WIDTH: OnceLock<Option<usize>> = OnceLock::new();
	TARGET_POINTER_WIDTH
		.get_or_init(|| {
			from_cargo!("CARGO_CFG_TARGET_POINTER_WIDTH")
				.map(|val| val.into_string().unwrap().parse().unwrap())
		})
		.as_ref()
}

enum COptsConfig
{
	InitCode,
	Normal
}

fn compile_c_code(
	infile: &str,
	outfile: impl AsRef<Path>,
	config: COptsConfig,
	additional_params: &[String]
)
{
	let mut target_triple = get_target_arch().unwrap().to_owned();
	if target_triple.ends_with("-none")
	{
		target_triple.push_str("-elf");
	}
	let target_cpu = get_target_cpu().unwrap();
	let optimizations = if get_profile().unwrap().contains("lto") || *get_opt_lvl().unwrap() != 0
	{
		vec!["-flto", "-O3"]
	}
	else
	{
		vec!["-ggdb3", "-Og"]
	};
	let target_ptr_width = get_target_ptr_width().unwrap();
	let opts = match config
	{
		COptsConfig::InitCode =>
		{
			let default_cpu = if target_triple.starts_with("x86_64")
			{
				"x86-64"
			}
			else
			{
				"generic"
			};
			// TODO: "-msoft-float" vs "-mfloat-abi=soft"
			//       is dependent on the target arch
			vec![
				format!("-march={default_cpu}"),
				format!("-mtune={default_cpu}"),
				"-mgeneral-regs-only".into(),
				"-mno-mmx".into(),
				"-mno-sse".into(),
				"-mno-sse2".into(),
				"-mno-red-zone".into(),
				"-mno-avx".into(),
				"-mno-avx2".into(),
				"-mno-avx512f".into(),
				//"-nostartfiles",
				//"-m128bit-long-double",
				//"-mfloat-abi=soft",
				"-mno-fp-ret-in-387".into(),
				"-msoft-float".into(),
			]
		},
		COptsConfig::Normal =>
		{
			vec![
				format!("-march={target_cpu}"),
				format!("-mtune={target_cpu}"),
			]
		}
	};
	Command::new("clang")
		.args(optimizations)
		.args(opts)
		.args(additional_params)
		.args([
			"-I./include",
			format!("--target={target_triple}").as_ref(),
			"-xc",
			"-std=gnu23",
			"-Wall",
			"-Wextra",
			"-Werror",
			"-ffreestanding",
			"-fno-stack-protector",
			"-fno-stack-check",
			"-fcolor-diagnostics",
			format!("-m{target_ptr_width}").as_ref(),
			"-masm=att",
			"-mcmodel=kernel",
			"-nodefaultlibs",
			"-nostdlib",
			//"-nostartfiles",
			//"-m128bit-long-double",
			//"-mfloat-abi=soft",
			// "-msoft-float",
			"-c"
		])
		.arg(infile)
		.arg("-o")
		.arg(outfile.as_ref())
		.spawn()
		.expect("couldn't spawn Clang !")
		.wait()
		.expect("couldn't compile C file !")
		.exit_ok()
		.expect("couldn't compile C file !");
}

fn compile_region_allocator() -> Vec<PathBuf>
{
	// clang -march=x86-64 -O3 -ffreestanding -fno-builtin -nostdlib
	// --target=x86_64-unknown-none-elf zerOS/src/kernel/memory/allocators/region.c
	// -c -mfloat-abi=soft
	// -Wall -Wextra -std=gnu23 	-xc -ffreestanding -fno-stack-protector
	// -fno-stack-check -m64 -mno-mmx 	$(call CC_TUNE_FOR,$(KCPU))	-mno-sse
	// -mno-sse2 	-mno-red-zone -mno-avx -mno-avx2 -mno-avx512f 	-nodefaultlibs
	// -nostdlib -nostartfiles 	-m128bit-long-double -fno-lto -msoft-float
	// and maybe `-fno-builtin` when compiling the four aformentioned `mem...`
	// functions
	let outdir = get_outdir();
	if outdir.is_none()
	{
		return vec![];
	}

	let files = vec![
		(
			"./src/utils/rbtree.c",
			path::PathBuf::from(outdir.unwrap()).join("rbtree.o")
		),
		(
			"./src/kernel/memory/allocators/region.c",
			path::PathBuf::from(outdir.unwrap()).join("region.o")
		),
	];

	for (inf, outf) in &files
	{
		compile_c_code(&inf, &outf, COptsConfig::Normal, &[]);
	}

	let mut target_triple = get_target_arch().unwrap().to_owned();
	if target_triple.ends_with("-none")
	{
		target_triple.push_str("-elf");
	}
	let target_ptr_width = get_target_ptr_width().unwrap();
	bindgen::builder()
		.clang_args([
			"-I./include",
			format!("--target={target_triple}").as_ref(),
			"-xc",
			"-std=gnu23",
			"-O3",
			"-Wall",
			"-Wextra",
			// "-flto",
			"-ffreestanding",
			"-fno-stack-protector",
			"-fno-stack-check",
			format!("-m{target_ptr_width}").as_ref(),
			"-masm=att",
			"-nodefaultlibs",
			"-nostdlib"
		])
		.header("./include/zerOS/region_allocator.h")
		.opaque_type("zerOS_region_allocator")
		.newtype_enum("zerOS_allocation_strategy")
		.prepend_enum_name(false)
		.allowlist_file(r"\./include/.+\.h")
		.blocklist_function(r"mem(cpy|move|set)")
		.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
		.rustfmt_configuration_file(Some("./rustfmt.toml".into()))
		.use_core()
		.generate()
		.unwrap()
		.write_to_file("./src/kernel/memory/allocators/bindings/region.rs")
		.unwrap();
	files.iter().cloned().unzip::<_, _, Vec<_>, _>().1
}

macro_rules! custom_kcfg {
	($cfg:ident : $type:ty = $parsed:expr) => {
		to_cargo!("rustc-cfg" => format!("{}=\"{}\"", stringify!($cfg), $parsed));
		let mut cfgstr = String::from(format!("cfg({}, values(", stringify!($cfg)));
		cfgstr += format!("\"{}\"", <$type>::VARIANTS[0]).as_str();
		for authorized in <$type>::VARIANTS.iter().skip(1)
		{
			cfgstr += format!(",\"{}\"", authorized).as_str();
		}
		cfgstr += "))";
		to_cargo!("rustc-check-cfg" => cfgstr);
	};
}

fn generate_kconfig_aliases()
{
	let kconfig = parse_kconfig();
	let bootconf = &kconfig.boot;
	let bootloader = bootconf.bootloader;
	custom_kcfg!(bootloader: KConfigBootBootloader = bootloader.as_ref());
}

fn compile_c_init_code() -> Vec<PathBuf>
{
	let kconfig = parse_kconfig();
	let defines = [format!(
		"-DzerOS_INIT_BOOTLOADER_IS_{}=1",
		kconfig
			.boot
			.bootloader
			.as_ref()
			.to_uppercase()
			.replace("-", "_")
	)];

	let outdir = get_outdir();
	if outdir.is_none()
	{
		return vec![];
	}

	let arch_dir = get_target_arch().unwrap().split('-').collect::<Vec<_>>()[0]
		.replace("x86-64", "amd64")
		.replace("x86_64", "amd64")
		.replace("arm64", "aarch64");

	let outf = path::PathBuf::from(outdir.unwrap()).join("entry-point.o");
	compile_c_code(
		format!("./src/arch/{arch_dir}/entry-point.c").as_ref(),
		&outf,
		COptsConfig::InitCode,
		&defines
	);
	vec![outf]
}

fn generate_config_arch_aliases()
{
	cfg_aliases! {
		x86_alike: { any(target_arch = "x86", target_arch = "x86_64") },
		avr_alike: { target_arch = "avr" },
		sparc_alike: { any(target_arch = "sparc", target_arch = "sparc64") },
		loongarch_alike: { target_arch = "loongarch64" },
		mips_alike: { any(
			target_arch = "mips",
			target_arch = "mips64",
			target_arch = "mips32r6",
			target_arch = "mips64r6") },
		ppc_alike: { any(target_arch = "powerpc", target_arch = "powerpc64") },
		riscv_alike: { any(target_arch = "riscv32", target_arch = "riscv64") },
		arm_alike: { any(target_arch = "aarch64", target_arch = "arm", target_arch = "arm64ec") },
		zarch_alike: { any(target_arch = "s390", target_arch = "s390x") }
		// TODO: IA64, Alpha DEC, SuperH, OpenRISC (?), C-Sky, HPPA (?)
	};
}

fn declare_c_source_code_in<T: AsRef<path::Path>>(paths: &[T], recurse: bool)
{
	for path in paths
	{
		for in_dir in fs::read_dir(path).unwrap()
		{
			if let Ok(p) = in_dir
			{
				if p.file_type().map_or(false, |val| {
					val.is_file()
						&& p.file_name().into_string().map_or(false, |filename| {
							filename.ends_with(".c") || filename.ends_with(".h")
						})
				})
				{
					to_cargo!(
						"rerun-if-changed" => p.file_name().into_string().unwrap()
					);
				}
				else if recurse && p.file_type().map_or(false, |val| val.is_dir())
				{
					declare_c_source_code_in(&[p.path()], recurse);
				}
			}
		}
	}
}

pub fn main()
{
	generate_config_arch_aliases();
	generate_kconfig_aliases();
	declare_c_source_code_in(&["./include", "./src"], true);

	let relpath: &'static str = "../scripts/gensectioninfo.py";
	let abspath = match realpath(relpath)
	{
		Ok(path) => path,
		Err(e) => panic!("can not find {relpath}: {}", e.to_string())
	};
	let out_dir = get_outdir();

	to_cargo!("rerun-if-changed" => abspath
		.clone()
		.into_os_string()
		.into_string()
		.expect("invalid path !"));
	to_cargo!("rerun-if-changed" => "build.rs");
	to_cargo!("rerun-if-changed" => "linker/linker-x86_64.ld.template");

	let mut c_objs = vec![];
	// TODO: change clang target based on target arch
	c_objs.append(&mut compile_region_allocator());
	// TODO: change clang target based on target arch
	c_objs.append(&mut compile_c_init_code());

	if let Some(odir) = out_dir
	{
		make_lib_with(&c_objs, &PathBuf::from(odir).join("libzerOS-c.a"));
	}

	let linker_script = update_linker_script_and_related(&abspath)
		.into_os_string()
		.into_string()
		.expect("unreachable");
	to_cargo!("rustc-link-arg" => format!("-T{linker_script}"));
}

fn realpath<P: AsRef<std::path::Path> + Clone>(path: P) -> io::Result<std::path::PathBuf>
{
	let thispath: &std::path::Path = ".".as_ref();
	fs::canonicalize(thispath.join(path))
}

macro_rules! KERNEL_SECTION_LIST {
	() => {
		KERNEL_SECTION_LIST!(identity_expand)
	};
	($callback:tt) => {
		callback!($callback([
			"text",
			"bootcode",
			"ctors_init_array",
			"rodata",
			"data",
			"bss" // dynamic
		]))
	};
}

fn format_generated_file<P: AsRef<std::path::Path>>(path: P) -> io::Result<std::process::Child>
{
	let config_file_path = realpath("./rustfmt.toml").expect("couldn't find config file !");
	let config_file = config_file_path
		.as_os_str()
		.try_into()
		.expect("couldn't convert path to string !");
	let args = [
		"--config-path",
		config_file,
		path.as_ref()
			.as_os_str()
			.try_into()
			.expect("couldn't convert path to string !")
	];
	println!(
		"formatting {} with command `{} {}`",
		&args[2],
		&config_file,
		args.join(" ")
	);
	Command::new("rustfmt").args(args).spawn()
}

const KERNEL_SECTION_COUNT: usize = KERNEL_SECTION_LIST!(array_size);
const KERNEL_SECTIONS: [&str; KERNEL_SECTION_COUNT] = KERNEL_SECTION_LIST!();

fn write_map_mod_file(filepath: &PathBuf) -> Result<(), std::io::Error>
{
	let mut out = std::fs::File::create(filepath)?;
	// let mut content: String = "\nmod __generated;\n\n".into();
	let mut content: String = r#"
pub use super::__generated::__linker_symbols::zerOS_section_count;

pub use super::__generated::__linker_symbols::zerOS_kernel_start;
pub use super::__generated::__linker_symbols::zerOS_kernel_end;

"#
	.into();
	for section in KERNEL_SECTIONS
	{
		content += format!(
			"pub use super::__generated::__linker_symbols::zerOS_{}_start;\n",
			section
		)
		.as_str();
		content += format!(
			"pub use super::__generated::__linker_symbols::zerOS_{}_end;\n",
			section
		)
		.as_str();
		content += format!(
			"pub use super::__generated::__linker_symbols::zerOS_{}_size;\n\n",
			section
		)
		.as_str();
	}
	out.write_all(content.as_bytes())?;
	Ok(())
}

fn make_lib_with(files: &Vec<PathBuf>, outlib: &PathBuf)
{
	let search_dir = outlib.parent().unwrap().to_str().unwrap();

	let lib_name = outlib.file_name().unwrap().to_str().unwrap();
	let lib_name = lib_name.strip_prefix("lib").unwrap_or(lib_name);
	let lib_name = lib_name.strip_suffix(".a").unwrap_or(lib_name);

	to_cargo!(
		"rustc-link-search" => search_dir
	);
	to_cargo!(
		"rustc-link-lib" =>
			"static=".to_owned() + lib_name
	);

	Command::new("ar")
		.arg("-rcv")
		.arg(outlib)
		.args(files)
		.spawn()
		.expect("couldn't spawn ar !")
		.wait()
		.expect("couldn't create an archive !")
		.exit_ok()
		.expect("couldn't create an archive !");
}

fn update_linker_script_and_related(gensecinfo: &PathBuf) -> std::path::PathBuf
{
	let relrsfile = "./src/kernel/linker/map";
	let modrsfile: PathBuf;
	let rsfile: OsString = match realpath(relrsfile)
	{
		Ok(path) =>
		{
			modrsfile = path.join("public_generated.rs");
			path.join("__generated.rs").into_os_string()
		},
		Err(e) => panic!("can not find {}: {}", relrsfile, e.to_string())
	};
	let rsfile: String = rsfile.into_string().expect("invalid path string");
	let relldfiles = ["./linker/linker-x86_64.ld.template", "./linker"];
	let (in_ldfile, out_ldfile) = match relldfiles.map(realpath)
	{
		[Ok(pathin), Ok(pathout)] => (pathin, pathout.join("linker-x86_64.ld")),
		[Err(e), _] => panic!("can not find {}: {}", relldfiles[0], e.to_string()),
		[_, Err(e)] => panic!("can not find {}: {}", relldfiles[1], e.to_string())
	};

	let params: [&String; 6] = [
		&"-r".into(),
		&rsfile,
		&"-l".into(),
		&out_ldfile.to_str().expect("invalid path").into(),
		&"-i".into(),
		&in_ldfile.to_str().expect("invalid path").into()
	];
	let ld_script = Command::new(gensecinfo)
		.args(params)
		.args(KERNEL_SECTIONS)
		.status()
		.unwrap_or_else(|_| {
			panic!(
				"unable to generate {:?} and {:?} from template {:?}",
				out_ldfile, rsfile, in_ldfile
			)
		});
	if !ld_script.success()
	{
		panic!(
			"unable to generate {:?} and {:?} from template {:?}",
			out_ldfile, rsfile, in_ldfile
		);
	}
	println!(
		"successfully generated {:?} and {:?} from template {:?}",
		out_ldfile, rsfile, in_ldfile
	);
	write_map_mod_file(&modrsfile)
		.map_err(|err| -> Result<(), std::io::Error> {
			panic!(
				"couldn't write file {}: {}",
				modrsfile.to_str().expect("invalid string"),
				err.to_string()
			)
		})
		.expect("unreachable"); // we `panic!`
	if let Ok(mut process) = format_generated_file(&modrsfile)
	{
		if process.wait().is_ok_and(|status| status.success())
		{
			println!("successfully formatted {}", modrsfile.display());
		}
		else
		{
			println!("couldn't properly format {}", modrsfile.display());
		}
	}
	else
	{
		println!("couldn't properly format {}", modrsfile.display());
	}

	if let Ok(mut process) = format_generated_file(&rsfile)
	{
		if process.wait().is_ok_and(|status| status.success())
		{
			println!("successfully formatted {}", rsfile);
		}
		else
		{
			println!("couldn't properly format {}", rsfile);
		}
	}
	else
	{
		println!("couldn't properly format {}", rsfile);
	}

	out_ldfile
}
