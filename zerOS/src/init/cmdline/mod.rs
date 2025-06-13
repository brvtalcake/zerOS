use alloc::{borrow::Cow, string::String};
use core::marker::{self, PhantomCovariantLifetime};

use anyhow::{Ok, Result, bail};

mod lex;
mod parse;

use lex::{LexerError, SpannedLexer, Token};
use parse::ParsedCmdlineOption;
use phf::phf_map;
use unicase::UniCase;

use crate::{error, init::cmdline::parse::ParsedCmdlineValue, kernel::sync::BasicRwLock};

pub struct KernelCmdline<'source>
{
	pub log_level: log::LevelFilter,
	_marker:       marker::PhantomCovariantLifetime<'source>
}

impl<'source> Default for KernelCmdline<'source>
{
	fn default() -> Self
	{
		Self::new()
	}
}

#[overloadf::overload]
fn map_loglvl(lvl: i128) -> log::LevelFilter
{
	match lvl
	{
		0 => log::LevelFilter::Off,
		1 => log::LevelFilter::Error,
		2 => log::LevelFilter::Warn,
		3 => log::LevelFilter::Info,
		4 => log::LevelFilter::Debug,
		5 => log::LevelFilter::Trace,
		_ => panic!("invalid integer log level specified in kernel command line: {lvl}")
	}
}

#[overloadf::overload]
fn map_loglvl(lvl: &String) -> log::LevelFilter
{
	match lvl
		.to_uppercase()
		.chars()
		.take(3)
		.collect::<heapless::String<{ 3 * 4 }>>()
		.as_str()
	{
		"OFF" => log::LevelFilter::Off,
		"ERR" => log::LevelFilter::Error,
		"WAR" => log::LevelFilter::Warn,
		"INF" => log::LevelFilter::Info,
		"DEB" => log::LevelFilter::Debug,
		"TRA" => log::LevelFilter::Trace,
		_ => panic!("invalid log level specified in kernel command line: \"{lvl}\"")
	}
}

#[overloadf::overload]
fn map_loglvl<'source>(lvl: &Cow<'source, str>) -> log::LevelFilter
{
	match lvl
		.to_uppercase()
		.chars()
		.take(3)
		.collect::<heapless::String<{ 3 * 4 }>>()
		.as_str()
	{
		"OFF" => log::LevelFilter::Off,
		"ERR" => log::LevelFilter::Error,
		"WAR" => log::LevelFilter::Warn,
		"INF" => log::LevelFilter::Info,
		"DEB" => log::LevelFilter::Debug,
		"TRA" => log::LevelFilter::Trace,
		_ => panic!("invalid log level specified in kernel command line: \"{lvl}\"")
	}
}

#[overloadf::overload]
fn map_loglvl(lvl: f64) -> log::LevelFilter
{
	let int_lvl = lvl as i128;
	if int_lvl < 0 || int_lvl > 5
	{
		panic!("invalid floating-point log level specified in kernel command line: {lvl}")
	}
	map_loglvl(int_lvl)
}

impl<'source> KernelCmdline<'source>
{
	const fn new() -> Self
	{
		const DEFAULT_LOG_LEVEL: log::LevelFilter = const {
			if cfg!(any(test, debug_assertions))
			{
				log::LevelFilter::Trace
			}
			else
			{
				log::LevelFilter::Info
			}
		};
		Self {
			log_level: DEFAULT_LOG_LEVEL,
			_marker: PhantomCovariantLifetime::new()
		}
	}

	fn maybe_update(&mut self, parsed: &ParsedCmdlineOption<'source>) -> Result<()>
	{
		const FN_MAP: phf::Map<
			UniCase<&'static str>,
			&'static dyn for<'input> Fn(
				&mut KernelCmdline<'input>,
				Option<&ParsedCmdlineValue<'input>>
			) -> bool
		> = phf_map! {
			UniCase::ascii("LogLvl") => &maybe_loglvl,
			UniCase::ascii("LogLevel") => &maybe_loglvl,
			UniCase::ascii("Log_Lvl") => &maybe_loglvl,
			UniCase::ascii("Log_Level") => &maybe_loglvl,
			UniCase::ascii("Log-Lvl") => &maybe_loglvl,
			UniCase::ascii("Log-Level") => &maybe_loglvl,
		};

		if let Some(&func) = FN_MAP.get(&UniCase::new(parsed.name.as_ref()))
		{
			if func(self, parsed.value.as_ref())
			{
				Ok(())
			}
			else
			{
				if let Some(ref arg) = parsed.value
				{
					bail!(
						"unknown argument to command-line option \"{}\": \"{}\"",
						parsed.name,
						arg
					)
				}
				else
				{
					bail!(
						"unknown argument to command-line option \"{}\": <none>",
						parsed.name
					)
				}
			}
		}
		else
		{
			bail!("unknown command-line option: \"{}\"", parsed.name)
		}
	}
}

impl<'source> From<&'source str> for KernelCmdline<'source>
{
	fn from(value: &'source str) -> Self
	{
		let lexer = SpannedLexer::new(value);
		let parser = parse::CmdlineParser::new();
		let parsed = parser.parse(value, lexer).unwrap_or_else(|err| {
			panic!(
				"couldn't parse command-line arguments: {}",
				err.map_error(|inner| inner)
			)
		});
		let mut kcmdline = KernelCmdline::new();
		for opt in parsed.iter()
		{
			if let Err(err) = kcmdline.maybe_update(opt)
			{
				error!(event: "command-line", "command-line option ignored: {err}");
			}
		}
		kcmdline
	}
}

fn maybe_loglvl<'source>(
	this: &mut KernelCmdline<'source>,
	parsed: Option<&ParsedCmdlineValue<'source>>
) -> bool
{
	if let Some(&ParsedCmdlineValue::Integer(i)) = parsed
	{
		let mapped = map_loglvl(i);
		this.log_level = mapped;
		true
	}
	else if let Some(&ParsedCmdlineValue::Float(f)) = parsed
	{
		let mapped = map_loglvl(f);
		this.log_level = mapped;
		true
	}
	else if let Some(ParsedCmdlineValue::String(s)) = parsed
	{
		let mapped = map_loglvl(s);
		this.log_level = mapped;
		true
	}
	else if let Some(ParsedCmdlineValue::Ident(s)) = parsed
	{
		let mapped = map_loglvl(s);
		this.log_level = mapped;
		true
	}
	else
	{
		false
	}
}

pub static ZEROS_COMMAND_LINE: BasicRwLock<KernelCmdline> = BasicRwLock::new(KernelCmdline::new());
