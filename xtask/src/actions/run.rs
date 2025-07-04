use chumsky::IterParser;
use clap::{ArgAction, Subcommand, ValueEnum};
use itertools::Itertools;
use tokio::{process, task};

use crate::{
	Endianness,
	SupportedArch,
	XtaskGlobalOptions,
	actions::{
		Xtask,
		configure::{Executable, ZerosConfig, get_topdir}
	},
	doc_comments::subdir,
	tools::{CmdIn, check, rm}
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Subcommand)]
#[clap(rename_all = "lowercase")]
pub(crate) enum XtaskRunnableSubproj
{
	#[doc = subdir!(zerOS)]
	#[clap(name = "zerOS", alias("zeros"), rename_all = "kebab-case")]
	#[clap(about = subdir!(zerOS))]
	Zeros
	{
		#[arg(short, long, value_enum, default_value_t)]
		/// The emulator to run zerOS on
		emulator: Emulator,

		#[arg(short = 'r', long, visible_alias("accel"), value_enum, default_value_t)]
		/// The possible accelerator to use
		accelerator: Accelerator,

		#[arg(short, long, value_enum, default_value_t)]
		/// The architecture to run zerOS on
		arch: SupportedArch,

		#[arg(short, long, default_value = "host", alias("mcu"))]
		/// The CPU/MCU targetted by zerOS (alias: --mcu)
		cpu: String,

		#[arg(short, long, default_value_t = num_cpus::get())]
		/// The number of CPUs to emulate
		num_cpus: usize,

		#[arg(short, long, default_value_t = false)]
		/// Whether to spawn a new terminal for debugging purposes
		gdb_window: bool,

		#[arg(short = 'm', long = "ram", default_value = "1500M")]
		memory: String,

		#[arg(long = "no-initial-wait", default_value_t = true, action = ArgAction::SetFalse)]
		initial_wait: bool,

		/// Arguments forwarded to the chosen emulator
		emulator_args: Vec<String>
	},

	#[doc = subdir!(unwindtool)]
	#[clap(alias("unwind-tool"))]
	#[clap(about = subdir!(unwindtool))]
	UnwindTool
	{
		args: Vec<String>
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ValueEnum, Default)]
#[clap(rename_all = "lowercase")]
pub(crate) enum Emulator
{
	#[default]
	Qemu,
	Bochs
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ValueEnum, Default)]
#[clap(rename_all = "lowercase")]
pub(crate) enum Accelerator
{
	None,
	#[default]
	Kvm
}

#[derive(Default)]
enum Maybe<T>
{
	Some(T),
	#[default]
	None
}

impl<T> Maybe<Maybe<T>>
{
	pub(crate) fn flatten(self) -> Maybe<T>
	{
		match self
		{
			Self::Some(inner) => inner,
			_ => <Maybe<T>>::None
		}
	}
}

impl<T> Maybe<T>
{
	pub(crate) fn unwrap(self) -> T
	{
		match self
		{
			Self::Some(val) => val,
			_ => panic!("tried to unwrap a None value")
		}
	}
}

impl<T> std::ops::BitOr<Self> for Maybe<T>
{
	type Output = Self;

	fn bitor(self, rhs: Self) -> Self::Output
	{
		match self
		{
			Self::None => rhs,
			Self::Some(_) => self
		}
	}
}

impl<T> std::ops::BitOr<T> for Maybe<T>
{
	type Output = Self;

	fn bitor(self, rhs: T) -> Self::Output
	{
		match self
		{
			Self::None => Self::Some(rhs),
			Self::Some(_) => self
		}
	}
}

impl<T> chumsky::container::Container<T> for Maybe<T>
{
	fn with_capacity(_n: usize) -> Self
	{
		Default::default()
	}

	fn push(&mut self, item: T)
	{
		match self
		{
			Self::None =>
			{
				*self = Self::Some(item);
			},
			Self::Some(_) => unreachable!()
		}
	}
}

fn maybe<'src, I: chumsky::prelude::Input<'src>, O, E: chumsky::extra::ParserExtra<'src, I>, T>(
	parser: T
) -> chumsky::combinator::Collect<chumsky::combinator::Repeated<T, O, I, E>, O, Maybe<O>>
where
	T: chumsky::Parser<'src, I, O, E>
{
	parser
		.repeated()
		.at_least(0)
		.at_most(1)
		.collect::<Maybe<_>>()
}

fn parse_qemu_mem(input: String) -> String
{
	use chumsky::prelude::*;
	#[derive(Clone, Copy)]
	struct Exponent(u64);
	#[derive(Clone, Copy)]
	struct Base(u64);

	let basic_mult_choice = choice((
		just::<_, _, chumsky::extra::Default>('T').to(Exponent(4)),
		just::<_, _, chumsky::extra::Default>('G').to(Exponent(3)),
		just::<_, _, chumsky::extra::Default>('M').to(Exponent(2)),
		just::<_, _, chumsky::extra::Default>('K').to(Exponent(1))
	));

	let extended_mult_choice = maybe(
		maybe(just('i').to(Base(1024)))
			.then_ignore(just('B'))
			.map(|val| val | Base(1000))
	)
	.map(|val| val.flatten() | Base(1000));

	let suffix = just('B').to(1u64).or(maybe(
		basic_mult_choice
			.then(extended_mult_choice)
			.map(|val| (val.0, val.1.unwrap()))
	)
	.map(|val| {
		match val
		{
			Maybe::Some((Exponent(exp), Base(base))) =>
			{
				let mut mult = 1;
				for _ in 0..exp
				{
					mult *= base;
				}
				mult
			},
			// there was no suffix
			Maybe::None => 1
		}
	}));

	let parser = number::<{ chumsky::number::format::STANDARD }, _, _, _>()
		.then(suffix)
		.map(|(num, mult): (u64, u64)| num * mult);

	let parsed = parser.parse(input.trim());

	let ram: u64 = ((parsed.unwrap() as f64) / (1000f64 * 1000f64)).round() as u64;

	let mut buf = itoa::Buffer::new();
	let mut ret = buf.format(ram).to_owned();
	ret.push('M');
	ret
}

#[allow(non_snake_case)]
fn run_zerOS_qemu_cmd(
	globals: &XtaskGlobalOptions,
	emulator: &Emulator,
	accelerator: &Accelerator,
	arch: &SupportedArch,
	cpu: &String,
	num_cpus: &usize,
	gdb_window: &bool,
	memory: &String,
	initial_wait: &bool,
	emulator_args: &Vec<String>
) -> CmdIn
{
	let cfg = ZerosConfig::load_or_error();
	let mut buf = itoa::Buffer::new();
	let gdb_stub = globals.debug || *gdb_window;

	match emulator
	{
		Emulator::Qemu =>
		{
			let mut qemu_args = emulator_args.iter().map(|s| s.as_str()).collect_vec();

			let mem = parse_qemu_mem(memory.clone());
			qemu_args.extend_from_slice(&[
				"-fw_cfg",
				"name=opt/dev.nullware.zerOS,string=testing=0",
				"-m",
				mem.as_str(),
				"-smp",
				buf.format(*num_cpus),
				"-cpu",
				cpu.as_str(),
				"-smbios",
				"type=0,uefi=on",
				"-device",
				"isa-debug-exit,iobase=0xf4,iosize=0x04",
				"-debugcon",
				"file:debugcon.log",
				"-serial",
				"stdio"
			]);

			if matches!(accelerator, Accelerator::Kvm)
			{
				// qemu_args.extend_from_slice(&["-accel", "kvm"]);
				qemu_args.extend_from_slice(&["-enable-kvm", "-accel", "kvm"]);
			}

			if gdb_stub || *initial_wait
			{
				qemu_args.push("-S");
			}

			qemu_args.extend_from_slice(&[
				"-bios",
				"zerOS/vendor/OVMF.fd",
				"-cdrom",
				"./zerOS/bin/zerOS.iso"
			]);

			if gdb_stub
			{
				qemu_args.push("-s");
			}

			let mut cmd = process::Command::new(cfg.get(&Executable::Qemu(
				*arch,
				Endianness::qemu_default_for(*arch)
			)));
			cmd.args(qemu_args);

			CmdIn::new(get_topdir(), cmd)
		},
		Emulator::Bochs => todo!()
	}
}

impl Xtask for XtaskRunnableSubproj
{
	async fn execute(&self, globals: &XtaskGlobalOptions)
	{
		let mut spawned = vec![];
		match self
		{
			Self::Zeros {
				emulator,
				accelerator,
				arch,
				cpu,
				num_cpus,
				gdb_window,
				memory,
				initial_wait,
				emulator_args
			} =>
			{
				rm(false, false, &get_topdir().join("debugcon.log")).await;
				spawned.push(task::spawn(
					run_zerOS_qemu_cmd(
						globals,
						emulator,
						accelerator,
						arch,
						cpu,
						num_cpus,
						gdb_window,
						memory,
						initial_wait,
						emulator_args
					)
					.finalize()
				));
				if *gdb_window
				{ /* todo!() */ }
			},
			Self::UnwindTool { .. } => todo!()
		}

		for task in spawned
		{
			check!(task.await.expect("process exited abnormally"));
		}
	}
}

#[cfg(test)]
mod tests
{
	extern crate test;

	use super::*;

	#[test]
	fn test_parse_qemu_mem()
	{
		assert_eq!(
			parse_qemu_mem("1213456789".to_string()),
			"1213M".to_string()
		);
		assert_eq!(
			parse_qemu_mem("1213456789B".to_string()),
			"1213M".to_string()
		);
		assert_eq!(
			parse_qemu_mem("1213456789K".to_string()),
			"1213457M".to_string()
		);
		assert_eq!(
			parse_qemu_mem("1213456789KiB".to_string()),
			"1242580M".to_string()
		);
	}
}
