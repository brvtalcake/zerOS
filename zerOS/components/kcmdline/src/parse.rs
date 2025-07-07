use lalrpop_util::lalrpop_mod;

mod ast
{
	use alloc::{borrow::Cow, string::String, vec::Vec};
	use core::fmt;

	pub enum ParsedCmdlineValue<'source>
	{
		Ident(Cow<'source, str>),
		String(String),
		Integer(i128),
		Float(f64)
	}

	impl<'source> fmt::Display for ParsedCmdlineValue<'source>
	{
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
		{
			match self
			{
				Self::Ident(string) => write!(f, "{string}"),
				Self::String(string) => write!(f, "{string}"),
				Self::Integer(int) => write!(f, "{int}"),
				Self::Float(float) => write!(f, "{float}")
			}?;
			Ok(())
		}
	}

	pub struct ParsedCmdlineOption<'source>
	{
		pub name:  Cow<'source, str>,
		pub value: Option<ParsedCmdlineValue<'source>>
	}

	pub type ParsedCmdline<'source> = Vec<ParsedCmdlineOption<'source>>;
}
lalrpop_mod!(grammar, "/init/cmdline/grammar.rs");

pub use ast::{ParsedCmdlineOption, ParsedCmdlineValue};
pub use grammar::CmdlineParser;
