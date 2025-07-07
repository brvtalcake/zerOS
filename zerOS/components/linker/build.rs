#![feature(exit_status_error)]

use std::{fs, io::Write, path::Path, process::Command};

use eager2::{eager, eager_macro};
use zerOS_buildrs_utils::{
	format::format_rust_file,
	get_scripts,
	get_topdir,
	linker::linker_script,
	to_cargo
};

pub fn main()
{
	let ldin = linker_script(true);
	let ldout = linker_script(false);
	let rsout = "./src/map/__generated.rs";
	let pubrsout = "./src/map/public_generated.rs";
	let rustfmt_cfg = get_topdir().join("rustfmt.toml");

	to_cargo!("rerun-if-changed" => get_scripts().join("gensectioninfo.py")
		.into_os_string()
		.into_string()
		.expect("invalid path !"));
	to_cargo!("rerun-if-changed" => "build.rs");
	to_cargo!("rerun-if-changed" => ldin.display());

	gen_section_info(ldin, ldout, rsout, pubrsout, rustfmt_cfg);
}

#[eager_macro]
macro_rules! KERNEL_SECTION_LIST {
	() => {
		[
			"text",
			"bootcode",
			"ctors_init_array",
			"rodata",
			"eh_frame_hdr",
			"eh_frame",
			"data",
			"bss" // dynamic
		]
	};
}

#[eager_macro]
macro_rules! array_size {
    ([]) => {
        0
    };
    ([$first:expr]) => {
        1
    };
    ([$first:expr, $($other:expr),+ $(,)?]) => {
        1 + array_size!([$($other),+])
    };
}

const KERNEL_SECTION_COUNT: usize = eager! { array_size!(KERNEL_SECTION_LIST!()) };
const KERNEL_SECTIONS: [&str; KERNEL_SECTION_COUNT] = KERNEL_SECTION_LIST!();

fn write_map_public_file(filepath: impl AsRef<Path>) -> Result<(), std::io::Error>
{
	let mut out = fs::File::create(filepath)?;
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

fn gen_section_info(
	inld: impl AsRef<Path>,
	outld: impl AsRef<Path>,
	outrs: impl AsRef<Path>,
	pubrs: impl AsRef<Path>,
	rustfmt_cfg: impl AsRef<Path>
)
{
	let mut args = [
		"-r",
		outrs.as_ref().to_str().unwrap(),
		"-l",
		outld.as_ref().to_str().unwrap(),
		"-i",
		inld.as_ref().to_str().unwrap()
	]
	.to_vec();
	args.extend(&KERNEL_SECTIONS);
	Command::new(get_scripts().join("gensectioninfo.py"))
		.args(&args)
		.spawn()
		.expect("could not spawn gensectioninfo.py")
		.wait()
		.expect("could not wait gensectioninfo.py")
		.exit_ok()
		.expect("gensectioninfo.py terminated abnormally");
	write_map_public_file(&pubrs).expect("could not write rust file");
	format_rust_file(rustfmt_cfg, pubrs).expect("could not format generated rust file")
}
