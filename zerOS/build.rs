use core::panic;
use std;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use proc_macro_utils::array_size;
use macro_utils::{callback, identity_expand};

pub fn main() {
    /* let ld_script = Command::new("echo") */
    /*     .arg("Hello world") */
    /*     .output() */
    /*     .expect("Failed to execute command"); */
    let relpath: &'static str = "../scripts/gensectioninfo.py";
    let abspath = match realpath(relpath) {
        Ok(path) => path,
        Err(e)   => panic!("can not find {relpath}: {}", e.to_string())
    };

    println!(
        "cargo::rerun-if-changed={}",
        abspath.clone().into_os_string()
            .into_string().expect("invalid path !")
    );
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=linker/linker-x86_64.ld.template");

    let linker_script = update_linker_script_and_related(&abspath).into_os_string()
        .into_string().expect("unreachable");
    println!("cargo::rustc-link-arg=-T{linker_script}");
}

fn realpath<P: AsRef<std::path::Path> + Clone>(path: P) -> io::Result<std::path::PathBuf>
{
    let thispath: &std::path::Path = ".".as_ref();
    return fs::canonicalize(thispath.join(path));
}

macro_rules! KERNEL_SECTION_LIST {
    () => {
        KERNEL_SECTION_LIST!(identity_expand)
    };
    ($callback:tt) => {
        callback!(
            $callback(
                [
                    "text", "bootcode", "ctors_init_array",
                    "rodata", "data",
                    "bss" // dynamic
                ]
            )
        )
    };
}

const KERNEL_SECTION_COUNT: usize = KERNEL_SECTION_LIST!(array_size);
const KERNEL_SECTIONS: [&'static str; KERNEL_SECTION_COUNT] = KERNEL_SECTION_LIST!();

fn write_map_mod_file(filepath: &PathBuf) -> Result<(), std::io::Error>
{
    let mut out = std::fs::File::create(filepath)?;
    //let mut content: String = "\nmod __generated;\n\n".into();
    let mut content: String = r#"
pub use super::__generated::__linker_symbols::zerOS_section_count;

pub use super::__generated::__linker_symbols::zerOS_kernel_start;
pub use super::__generated::__linker_symbols::zerOS_kernel_end;

"#.into();
    for section in KERNEL_SECTIONS
    {
        content += format!("pub use super::__generated::__linker_symbols::zerOS_{}_start;\n" , section).as_str();
        content += format!("pub use super::__generated::__linker_symbols::zerOS_{}_end;\n"   , section).as_str();
        content += format!("pub use super::__generated::__linker_symbols::zerOS_{}_size;\n\n", section).as_str();
    }
    content += r#"
#[cfg(test)]
mod tests
{
    use super::zerOS_bss_size;

    #[test]
    fn compiles()
    {
        let _test = *zerOS_bss_size;
    }
}
"#;
    out.write(content.as_bytes())?;
    Ok(())
}

fn update_linker_script_and_related(gensecinfo: &PathBuf) -> std::path::PathBuf
{
    let relrsfile = "./src/kernel/linker/map";
    let modrsfile: PathBuf;
    let rsfile: OsString = match realpath(relrsfile) {
        Ok(path) => {
            modrsfile = path.join("public_generated.rs");
            path.join("__generated.rs").into_os_string()
        },
        Err(e) => panic!("can not find {}: {}", relrsfile, e.to_string()),
    };
    let rsfile: String = rsfile.into_string().expect("invalid path string");
    let relldfiles = [ "./linker/linker-x86_64.ld.template", "./linker" ];
    let (in_ldfile, out_ldfile) = match relldfiles.map(realpath) {
        [Ok(pathin), Ok(pathout)] => { (pathin, pathout.join("linker-x86_64.ld")) },
        [Err(e)    , _          ] => panic!("can not find {}: {}", relldfiles[0], e.to_string()),
        [_         , Err(e)     ] => panic!("can not find {}: {}", relldfiles[1], e.to_string()),
    };

    let params: [&String; 6] = [
        &"-r".into(), &rsfile,
        &"-l".into(), &out_ldfile.to_str().expect("invalid path").into(),
        &"-i".into(), &in_ldfile.to_str().expect("invalid path").into(),
    ];
    let ld_script = Command::new(gensecinfo)
        .args(params).args(KERNEL_SECTIONS)
        .status().unwrap_or_else(
            |_| panic!(
                "unable to generate {:?} and {:?} from template {:?}",
                out_ldfile, rsfile, in_ldfile
            )
        );
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
    write_map_mod_file(&modrsfile).or_else(
        |err| -> Result<(), std::io::Error> {
            panic!(
                "couldn't write file {}: {}",
                modrsfile.to_str().expect("invalid string"), err.to_string()
            )
        }
    ).expect("unreachable"); // we `panic!`
    out_ldfile
}
