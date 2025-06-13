use alloc::{fmt, string::String};
use core::{
	num::{ParseFloatError, ParseIntError},
	str::FromStr
};

use itertools::Itertools;
use logos::{Lexer, Logos, SpannedIter};

fn parse_str_value<'source>(lex: &mut Lexer<'source, Token<'source>>) -> String
{
	lex.slice()
		.chars()
		.dropping(1)
		.dropping_back(1)
		.as_str()
		.replace(r#"\""#, "\"")
}

fn parse_float_value<'source>(
	lex: &mut Lexer<'source, Token<'source>>
) -> Result<f64, <f64 as FromStr>::Err>
{
	lex.slice().parse()
}

fn parse_integer_value<'source>(
	lex: &mut Lexer<'source, Token<'source>>
) -> Result<i128, <i128 as FromStr>::Err>
{
	let slice = lex.slice();
	if slice.starts_with("0x")
	{
		i128::from_str_radix(slice.chars().dropping(2).as_str(), 16)
	}
	else
	{
		slice.parse()
	}
}

impl From<ParseIntError> for LexerError
{
	fn from(value: <i128 as FromStr>::Err) -> Self
	{
		Self::BadIntegerLiteral(value)
	}
}

impl From<ParseFloatError> for LexerError
{
	fn from(value: <f64 as FromStr>::Err) -> Self
	{
		Self::BadFloatLiteral(value)
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum LexerError
{
	BadIntegerLiteral(<i128 as FromStr>::Err),
	BadFloatLiteral(<f64 as FromStr>::Err),
	#[default]
	UnknownError
}

impl fmt::Display for LexerError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			Self::BadFloatLiteral(err) => write!(f, "{err}"),
			Self::BadIntegerLiteral(err) => write!(f, "{err}"),
			Self::UnknownError => write!(f, "LexerError: unknown error")
		}?;
		Ok(())
	}
}

#[derive(Debug, Logos, PartialEq, Clone)]
#[logos(error = LexerError)]
#[logos(subpattern alpha = r"[a-zA-Z]")]
#[logos(subpattern digit = r"[0-9]")]
#[logos(subpattern integer = r"-?(0|[1-9][0-9]*)")]
#[logos(subpattern float = r"-?((0|[1-9][0-9]*)\.[0-9]+)")]
#[logos(subpattern hex_integer = r"-?0x(0|[1-9a-fA-F][0-9a-fA-F]*)")]
#[logos(subpattern escaped_quote = r#"\\""#)]
#[logos(subpattern string_value = r#""([^"]|(?&escaped_quote))*""#)]
pub enum Token<'source>
{
	#[regex(r"((?&alpha)|(?&digit)|_)((?&alpha)|(?&digit)|_|-)*")]
	Ident(&'source str),

	#[regex("((?&integer)|(?&hex_integer))", callback = parse_integer_value, priority = 3)]
	IntegerValue(i128),
	#[regex("(?&float)", callback = parse_float_value, priority = 4)]
	FloatValue(f64),
	#[regex("(?&string_value)", callback = parse_str_value)]
	StringValue(String),

	#[token("=")]
	Equality,
	#[regex(r"\s+", logos::skip)]
	Spaces
}

impl<'source> fmt::Display for Token<'source>
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			Self::StringValue(s) =>
			{
				write!(f, "\"{s}\"")
			},
			Self::Ident(s) =>
			{
				write!(f, "{s}")
			},
			Self::IntegerValue(int) =>
			{
				write!(f, "{int}")
			},
			Self::FloatValue(float) =>
			{
				write!(f, "{float}")
			},
			Self::Equality =>
			{
				write!(f, "=")
			},
			Self::Spaces =>
			{
				write!(f, " ")
			}
		}?;
		Ok(())
	}
}

pub struct SpannedLexer<'source>
{
	// instead of an iterator over characters, we have a token iterator
	token_stream: SpannedIter<'source, Token<'source>>
}

impl<'source> SpannedLexer<'source>
{
	pub fn new(input: &'source str) -> Self
	{
		// the Token::lexer() method is provided by the Logos trait
		Self {
			token_stream: Token::lexer(input).spanned()
		}
	}
}

impl<'source> Iterator for SpannedLexer<'source>
{
	type Item = Result<(usize, Token<'source>, usize), LexerError>;

	fn next(&mut self) -> Option<Self::Item>
	{
		self.token_stream
			.next()
			.map(|(token, span)| Ok((span.start, token?, span.end)))
	}
}
