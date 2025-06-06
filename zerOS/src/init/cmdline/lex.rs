use alloc::string::String;
use core::{
	num::{ParseFloatError, ParseIntError},
	str::FromStr
};

use itertools::Itertools;
use logos::{Lexer, Logos};

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
) -> Result<u128, <u128 as FromStr>::Err>
{
	let slice = lex.slice();
	if slice.starts_with("0x")
	{
		u128::from_str_radix(slice.chars().dropping(2).as_str(), 16)
	}
	else
	{
		slice.parse()
	}
}

impl From<ParseIntError> for LexerError
{
	fn from(value: <u128 as FromStr>::Err) -> Self
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
enum LexerError
{
	BadIntegerLiteral(<u128 as FromStr>::Err),
	BadFloatLiteral(<f64 as FromStr>::Err),
	#[default]
	UnknownError
}

#[derive(Debug, Logos, PartialEq)]
#[logos(error = LexerError)]
#[logos(subpattern alpha = r"[a-zA-Z]")]
#[logos(subpattern digit = r"[0-9]")]
#[logos(subpattern integer = r"(0|[1-9][0-9]*)")]
#[logos(subpattern float = r"((0|[1-9][0-9]*)\.[0-9]+)")]
#[logos(subpattern hex_integer = r"0x(0|[1-9a-fA-F][0-9a-fA-F]*)")]
#[logos(subpattern ident_value = r"(?&alpha)|(?&digit)|_")]
#[logos(subpattern escaped_quote = r#"\\""#)]
#[logos(subpattern string_value = r#""([^"]|(?&escaped_quote))*""#)]
enum Token<'a>
{
	#[regex("(?&ident_value)+")]
	IdentValue(&'a str),
	#[regex("((?&integer)|(?&hex_integer))", callback = parse_integer_value, priority = 3)]
	IntegerValue(u128),
	#[regex("(?&float)", callback = parse_float_value, priority = 4)]
	FloatValue(f64),
	#[regex("(?&string_value)", callback = parse_str_value)]
	StringValue(String),

	#[token("=")]
	Equality,
	#[regex(r"\s+", logos::skip)]
	Spaces
}

compile_error!("TODO: implement a parser!");