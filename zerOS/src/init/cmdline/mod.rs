use alloc::borrow::Cow;

use itertools::Itertools;
use limine::response::ExecutableCmdlineResponse;

mod lex;

pub struct KernelCmdline
{
	pub log_level: log::LevelFilter
}

fn next_arg<'a>(cmdline: Cow<'a, str>) -> (Cow<'a, str>, Cow<'a, str>, Cow<'a, str>)
{
	let mut chars = cmdline.chars().skip_while(|chr| chr.is_alphanumeric());
	let key = chars
		.take_while_ref(|chr| *chr != '=' && chr.is_alphanumeric())
		.collect();
	let curr = chars.next();
	let value = chars
		.skip(1)
		.take_while_ref(|chr| chr.is_alphanumeric())
		.collect();
	(key, value, chars.collect())
}

impl From<&ExecutableCmdlineResponse> for KernelCmdline
{
	fn from(value: &ExecutableCmdlineResponse) -> Self
	{
		let cmdline = value.cmdline().to_string_lossy();
		if let Some(idx) = cmdline.find("")
		{}
		Self { log_level: todo!() }
	}
}
