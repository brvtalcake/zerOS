use std::{fs, path::{Path, PathBuf}, process::Command};

use crate::{
	cargo::{get_opt_lvl, get_profile, get_target_arch, get_target_cpu, get_target_ptr_width},
	to_cargo
};

pub enum COptsConfig
{
	InitCode,
	Normal
}

pub fn compile_c_code(
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
			"-funwind-tables",
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

pub fn lalrpop_compile()
{
	lalrpop::process_src().unwrap();
}

pub fn declare_c_source_code_in<T: AsRef<Path>>(paths: &[T], recurse: bool)
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

pub fn make_lib_with(files: &Vec<PathBuf>, outlib: &PathBuf)
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
