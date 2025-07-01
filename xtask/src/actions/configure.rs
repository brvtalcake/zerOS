use std::{
	collections::HashMap,
	ffi::OsStr,
	fs::{self, File, OpenOptions},
	io::Write,
	mem::{self, MaybeUninit},
	path::PathBuf,
	str::FromStr,
	sync::{
		LazyLock,
		RwLock,
		atomic::{self, AtomicBool}
	}
};

use camino::{Utf8Path, Utf8PathBuf};
use clap::{Subcommand, ValueEnum};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use which::which;

use crate::{
	Endianness,
	SupportedArch,
	XtaskGlobalOptions,
	actions::Xtask,
	doc_comments::subdir,
	env,
	tools::check
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Subcommand)]
#[clap(rename_all = "lowercase")]
pub(crate) enum XtaskConfigurableSubproj
{
	#[doc = subdir!(zerOS)]
	#[clap(name = "zerOS", alias("zeros"), rename_all = "kebab-case")]
	#[clap(about = subdir!(zerOS))]
	Zeros
	{
		#[arg(short, long, value_enum, default_value_t)]
		bootloader:  KConfigBootBootloader,
		assignments: Vec<String>
	} // TODO: add Docs target
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum Executable
{
	Clang,
	Cargo,
	Xorriso,
	Strip,
	EuStrip,
	Objcopy,
	Qemu(SupportedArch, Option<Endianness>)
}

pub(crate) static EXECUTABLE_DEFAULTS: RwLock<
	MaybeUninit<HashMap<Executable, (&'static str, &'static [&'static str])>>
> = RwLock::new(MaybeUninit::uninit());

static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub(crate) fn init_default_executable_names()
{
	let mut guard = EXECUTABLE_DEFAULTS.write().unwrap();
	let map = guard.write(HashMap::with_capacity(
		mem::variant_count::<Executable>() + mem::variant_count::<SupportedArch>()
	));
	map.insert(Executable::Clang, ("clang", &["CC", "CLANG"]));
	map.insert(Executable::Cargo, ("cargo", &["CARGO"]));
	map.insert(Executable::Xorriso, ("xorriso", &["XORRISO"]));
	map.insert(Executable::Strip, ("strip", &["STRIP"]));
	map.insert(Executable::Objcopy, ("objcopy", &["OBJCOPY"]));
	map.insert(Executable::EuStrip, ("eu-strip", &["EU_STRIP", "EUSTRIP"]));
	map.insert(
		Executable::Qemu(SupportedArch::Amd64, None),
		(
			"qemu-system-x86_64",
			&[
				"QEMU",
				"QEMU_X86_64",
				"QEMUX86_64",
				"QEMUX64",
				"QEMU_X64",
				"QEMUAMD64",
				"QEMU_AMD64"
			]
		)
	);
	map.insert(
		Executable::Qemu(SupportedArch::AArch64, None),
		(
			"qemu-system-aarch64",
			&["QEMU", "QEMU_AARCH64", "QEMUAARCH64"]
		)
	);
	map.insert(
		Executable::Qemu(SupportedArch::Arm32, None),
		("qemu-system-arm", &["QEMU", "QEMU_ARM", "QEMUARM"])
	);
	map.insert(
		Executable::Qemu(SupportedArch::Avr32, None),
		("qemu-system-avr", &["QEMU", "QEMU_AVR", "QEMUAVR"])
	);
	map.insert(
		Executable::Qemu(SupportedArch::Riscv32, None),
		(
			"qemu-system-riscv32",
			&["QEMU", "QEMU_RISCV32", "QEMURISCV32"]
		)
	);
	map.insert(
		Executable::Qemu(SupportedArch::Riscv64, None),
		(
			"qemu-system-riscv64",
			&["QEMU", "QEMU_RISCV64", "QEMURISCV64"]
		)
	);
	map.insert(
		Executable::Qemu(SupportedArch::PowerPC32, None),
		("qemu-system-ppc", &["QEMU", "QEMU_PPC", "QEMUPPC"])
	);
	map.insert(
		Executable::Qemu(SupportedArch::PowerPC64, None),
		("qemu-system-ppc64", &["QEMU", "QEMU_PPC64", "QEMUPPC64"])
	);
	map.insert(
		Executable::Qemu(SupportedArch::Sparc32, None),
		("qemu-system-sparc", &["QEMU", "QEMU_SPARC", "QEMUSPARC"])
	);
	map.insert(
		Executable::Qemu(SupportedArch::Sparc64, None),
		(
			"qemu-system-sparc64",
			&["QEMU", "QEMU_SPARC64", "QEMUSPARC64"]
		)
	);
	map.insert(
		Executable::Qemu(SupportedArch::Mips32, Some(Endianness::Big)),
		("qemu-system-mips", &["QEMU", "QEMU_MIPS", "QEMUMIPS"])
	);
	map.insert(
		Executable::Qemu(SupportedArch::Mips64, Some(Endianness::Big)),
		("qemu-system-mips64", &["QEMU", "QEMU_MIPS64", "QEMUMIPS64"])
	);
	map.insert(
		Executable::Qemu(SupportedArch::Mips32, Some(Endianness::Little)),
		("qemu-system-mipsel", &["QEMU", "QEMU_MIPSEL", "QEMUMIPSEL"])
	);
	map.insert(
		Executable::Qemu(SupportedArch::Mips64, Some(Endianness::Little)),
		(
			"qemu-system-mips64el",
			&["QEMU", "QEMU_MIPS64EL", "QEMUMIPS64EL"]
		)
	);
	map.insert(
		Executable::Qemu(SupportedArch::LoongArch64, None),
		(
			"qemu-system-loongarch64",
			&["QEMU", "QEMU_LOONGARCH64", "QEMULOONGARCH64"]
		)
	);
	INITIALIZED.store(true, atomic::Ordering::Release);
}

pub(crate) fn get_default_executable_short_name(exe: &Executable) -> &'static str
{
	assert!(INITIALIZED.load(atomic::Ordering::Acquire));
	let locked = EXECUTABLE_DEFAULTS.read().unwrap();
	unsafe { locked.assume_init_ref() }.get(exe).unwrap().0
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct KConfig
{
	pub(crate) boot: KConfigBoot
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[serde(rename = "boot")]
pub(crate) struct KConfigBoot
{
	pub(crate) bootloader: KConfigBootBootloader
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
	Serialize,
	Deserialize,
	Debug,
	Default,
	Clone,
	Copy,
	PartialEq,
	Eq,
	Hash,
	ValueEnum,
	strum::AsRefStr,
	strum::EnumString,
	strum::VariantNames,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
#[serde(rename = "bootloader")]
pub(crate) enum KConfigBootBootloader
{
	#[default]
	Limine,
	GRUB2,
	UEFI
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct ZerosConfig
{
	#[serde_as(as = "Vec<(_, _)>")]
	pub(crate) executables: HashMap<Executable, Utf8PathBuf>,
	pub(crate) kcfg:        KConfig
}

impl ZerosConfig
{
	pub(crate) fn load_or_error() -> Self
	{
		let path = config_location!(zerOS);
		let file = check!(File::open(path).expect("could not read zerOS config file"));
		check!(
			file.lock_shared()
				.expect("could not lock zerOS config file")
		);
		check!(
			rmp_serde::decode::from_read(file).expect(
				"could not deserialize zerOS config file"
					=>
						"this might be a config format mismatch",
						"please re-run the configure step and file an issue if the error persists"
			)
		)
	}

	pub(crate) fn get(&self, name: &Executable) -> &Utf8Path
	{
		check!(
			self.executables
				.get(name)
				.ok_or("please (re-)run the configure step before retrying")
				.expect(
					format!(
						"could not find executable `{}`",
						get_default_executable_short_name(name)
					)
					.as_str()
				)
		)
	}
}

impl Xtask for XtaskConfigurableSubproj
{
	async fn execute(&self, globals: &XtaskGlobalOptions)
	{
		match self
		{
			Self::Zeros {
				bootloader,
				assignments
			} =>
			{
				let find_exe = |name: &'static str, alts: &[&'static str]| -> Option<Utf8PathBuf> {
					let raw = format!(r"({})\=(\S+)", alts.join("|"));
					let regex = Regex::new(raw.as_str()).unwrap();
					assignments
						.iter()
						.map(|el| regex.captures(el.trim()))
						.find(|el| el.as_ref().is_some_and(|matched| matched.get(2).is_some()))
						.flatten()
						.map(|matched| unsafe { matched.get(2).unwrap_unchecked() }.as_str().into())
						.or_else(|| {
							for env in alts.iter().map(|&alt| env::var(alt))
							{
								if env.is_some()
								{
									return env.map(Into::into);
								}
							}
							None
						})
						.or_else(|| {
							which(name)
								.ok()
								.and_then(|p| Utf8PathBuf::from_path_buf(p).ok())
						})
				};
				let mut cfg = ZerosConfig {
					executables: HashMap::new(),
					kcfg:        KConfig::default()
				};
				cfg.kcfg.boot.bootloader = *bootloader;

				let to_find = [
					Executable::Clang,
					Executable::Cargo,
					Executable::Xorriso,
					Executable::Strip,
					Executable::EuStrip,
					Executable::Objcopy,
					Executable::Qemu(SupportedArch::Amd64, None),
					Executable::Qemu(SupportedArch::AArch64, None),
					Executable::Qemu(SupportedArch::Arm32, None),
					Executable::Qemu(SupportedArch::Avr32, None),
					Executable::Qemu(SupportedArch::Riscv32, None),
					Executable::Qemu(SupportedArch::Riscv64, None),
					Executable::Qemu(SupportedArch::PowerPC32, None),
					Executable::Qemu(SupportedArch::PowerPC64, None),
					Executable::Qemu(SupportedArch::Sparc32, None),
					Executable::Qemu(SupportedArch::Sparc64, None),
					Executable::Qemu(SupportedArch::Mips32, Some(Endianness::Big)),
					Executable::Qemu(SupportedArch::Mips64, Some(Endianness::Big)),
					Executable::Qemu(SupportedArch::Mips32, Some(Endianness::Little)),
					Executable::Qemu(SupportedArch::Mips64, Some(Endianness::Little)),
					Executable::Qemu(SupportedArch::LoongArch64, None)
				];
				let guard = EXECUTABLE_DEFAULTS.read().unwrap();
				for (exe, name, vars) in to_find.map(|e| {
					let &(n, v) = unsafe { guard.assume_init_ref() }.get(&e).unwrap();
					(e, n, v)
				})
				{
					if let Some(found) = find_exe(name, vars)
					{
						cfg.executables.insert(exe, found);
					}
				}

				if globals.debug
				{
					dbg!(&cfg);
				}

				let mut file = OpenOptions::new()
					.create(true)
					.truncate(true)
					.write(true)
					.read(false)
					.open(config_location!(zerOS))
					.unwrap();
				file.lock().unwrap();
				rmp_serde::encode::write(&mut file, &cfg).unwrap()
			}
		}
	}
}

pub(crate) macro config_location
{
	() =>  {
		$crate::actions::configure::get_topdir().join(".xtask-cache")
	},
	(zerOS) => {
		$crate::actions::configure::config_location!().join("zerOS.msgpack")
	},
	($($errtoks:tt)*) => {
		compile_error!(concat!("invalid tokens: ", stringify!($($errtoks)*)));
	}
}

pub(crate) macro subproj_location($subdir:literal) {
	$crate::actions::configure::get_topdir().join($subdir)
}

pub(crate) fn get_topdir() -> &'static Utf8Path
{
	static TOPDIR: LazyLock<Utf8PathBuf> = LazyLock::new(|| {
		Utf8PathBuf::from_path_buf(PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap())
			.unwrap()
			.canonicalize_utf8()
			.unwrap()
			.parent()
			.unwrap()
			.to_path_buf()
	});
	TOPDIR.as_path()
}

// TODO: make it async
pub(crate) fn kcfg_write(kcfg: &KConfig)
{
	check!(
		toml::to_string_pretty(kcfg)
			.map_err(anyhow::Error::new)
			.and_then(|toml| {
				let mut file = OpenOptions::new()
					.create(true)
					.truncate(true)
					.write(true)
					.read(false)
					.open(subproj_location!("zerOS").join("kconfig.toml"))?;
				check!(file.lock().expect("could not lock file"));
				Ok(check!(
					file.write_all(toml.as_bytes())
						.expect("could not write to file")
				))
			})
			.expect("could not format kernel configuration")
	)
}
