#![allow(dead_code)]

use proc_macro_error2::{ResultExt as _, abort, emit_warning};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
	Expr,
	ExprCall,
	ExprLit,
	ExprPath,
	Lit,
	LitStr,
	Token,
	parenthesized,
	parse::{Parse, ParseBuffer, ParseStream, discouraged::Speculative as _},
	punctuated::Punctuated,
	spanned::Spanned as _,
	token::Paren
};

use crate::imp::JoinSpans as _;

mod kw
{
	use syn::custom_keyword;

	custom_keyword!(into);
	custom_keyword!(safety);
	custom_keyword!(bare);
	custom_keyword!(imp);
	custom_keyword!(interface);
	custom_keyword!(foreign);
	custom_keyword!(pre);
	custom_keyword!(precond);
	custom_keyword!(precondition);
	custom_keyword!(requires);
	custom_keyword!(ensures);
	custom_keyword!(post);
	custom_keyword!(postcond);
	custom_keyword!(postcondition);
	custom_keyword!(invariant);
}

#[derive(Debug)]
pub(super) enum UnsafeItemAttr
{
	SafetyOnly
	{
		lines: Punctuated<UnsafeItemSafetyElem, Token![,]>
	},
	Detailled
	{
		safety:    UnsafeItemAttrSafety,
		into:      Option<UnsafeItemAttrInto>,
		pre:       Vec<TokenStream2>,
		post:      Vec<TokenStream2>,
		invariant: Vec<TokenStream2>
	}
}

impl Parse for UnsafeItemAttr
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		let mut fork;

		fork = input.fork();
		if let Ok(lines) = safety_lines_from(&fork)
		{
			input.advance_to(&fork);
			return Ok(Self::SafetyOnly { lines });
		}

		fork = input.fork();
		let mut pre = vec![];
		let mut post = vec![];
		let mut invariant = vec![];
		let mut into = None;
		let mut safety = None;
		loop
		{
			if fork.is_empty()
			{
				break;
			}

			if fork.peek(kw::pre)
			{
				pre.push(parse_unsafe_check::<kw::pre>(&fork)?);
			}
			else if fork.peek(kw::precond)
			{
				pre.push(parse_unsafe_check::<kw::precond>(&fork)?);
			}
			else if fork.peek(kw::precondition)
			{
				pre.push(parse_unsafe_check::<kw::precondition>(&fork)?);
			}
			else if fork.peek(kw::requires)
			{
				pre.push(parse_unsafe_check::<kw::requires>(&fork)?);
			}
			else if fork.peek(kw::post)
			{
				post.push(parse_unsafe_check::<kw::post>(&fork)?);
			}
			else if fork.peek(kw::postcond)
			{
				post.push(parse_unsafe_check::<kw::postcond>(&fork)?);
			}
			else if fork.peek(kw::postcondition)
			{
				post.push(parse_unsafe_check::<kw::postcondition>(&fork)?);
			}
			else if fork.peek(kw::ensures)
			{
				post.push(parse_unsafe_check::<kw::ensures>(&fork)?);
			}
			else if fork.peek(kw::invariant)
			{
				invariant.push(parse_unsafe_check::<kw::invariant>(&fork)?);
			}
			else if fork.peek(kw::into)
			{
				if into.is_some()
				{
					return Err(syn::Error::new_spanned(
						fork.parse::<kw::into>()?,
						"parameter `into` can not be specified more than one time"
					));
				}
				into = Some(fork.parse()?);
			}
			else if fork.peek(kw::safety)
			{
				if safety.is_some()
				{
					return Err(syn::Error::new_spanned(
						fork.parse::<kw::safety>()?,
						"parameter `safety` can not be specified more than one time"
					));
				}
				safety = Some(fork.parse()?);
			}
			else if fork.peek(Token![,])
			{
				let _comma = fork.parse::<Token![,]>()?;
				continue;
			}
			else
			{
				return Err(syn::Error::new(
					fork.span(),
					format!(
						"expected one of `pre`, `post`, `invariant`, `into` or `safety`\ngot {} \
						 instead",
						fork.cursor().token_stream()
					)
				));
			}
		}
		if safety.is_none()
		{
			Err(syn::Error::new(
				input.span(),
				"you must provide a valid safety justification / documentation"
			))
		}
		else
		{
			input.advance_to(&fork);
			Ok(Self::Detailled {
				safety: unsafe { safety.unwrap_unchecked() },
				into,
				pre,
				post,
				invariant
			})
		}
	}
}

#[derive(Debug)]
pub(super) enum UnsafeItemSafetyOperator
{
	Include(String)
}

#[derive(Debug)]
pub(super) enum UnsafeItemSafetyElem
{
	String(LitStr),
	Func(UnsafeItemSafetyOperator)
}

impl UnsafeItemSafetyElem
{
	fn transform_func_call(call: ExprCall) -> syn::Result<Self>
	{
		if call.attrs.len() != 0
		{
			return Err(syn::Error::new(
				call.attrs.joined_span(),
				"not expecting any attributes here"
			));
		}
		let func_ident = match &*call.func
		{
			Expr::Path(ExprPath { attrs, qself, path }) =>
			{
				if attrs.len() != 0
				{
					return Err(syn::Error::new(
						attrs.joined_span(),
						"not expecting any attributes here"
					));
				}
				if let Some(qualified) = qself
				{
					return Err(syn::Error::new(
						qualified.span(),
						"can not handle this kind of path"
					));
				}
				path.require_ident()?
			},
			expr =>
			{
				return Err(syn::Error::new_spanned(expr, "unknown function"));
			}
		};
		Ok(
			match (
				func_ident.to_string().as_str(),
				&*call.args.iter().collect::<Vec<_>>()
			)
			{
				(
					"include" | "include_str",
					&[
						Expr::Lit(ExprLit {
							attrs,
							lit: Lit::Str(incpath)
						})
					]
				) =>
				{
					if attrs.len() != 0
					{
						emit_warning!(
							attrs.joined_span(), "not expecting any attributes here";
							note = "ignoring them"
						);
					}
					Self::Func(UnsafeItemSafetyOperator::Include(incpath.value()))
				},
				(actual_fn @ ("include" | "include_str"), other) =>
				{
					abort!(
						other.joined_span(), "unexpected arguments to documentation inclusion `{}`",
							actual_fn;
						note = "expected a string path instead"
					);
				},
				(actual_fn, _) =>
				{
					return Err(syn::Error::new_spanned(
						func_ident,
						format!("unknown function called: {actual_fn}")
					));
				}
			}
		)
	}
}

impl Parse for UnsafeItemSafetyElem
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		let mut fork;

		fork = input.fork();
		if let Ok(litstr) = fork.parse()
		{
			input.advance_to(&fork);
			return Ok(Self::String(litstr));
		}

		fork = input.fork();
		if let Ok(this) = fork.parse::<ExprCall>().and_then(Self::transform_func_call)
		{
			input.advance_to(&fork);
			return Ok(this);
		}

		Err(syn::Error::new(
			input.span(),
			"expected a string or a function call"
		))
	}
}

impl ToTokens for UnsafeItemSafetyElem
{
	fn to_tokens(&self, tokens: &mut TokenStream2)
	{
		match self
		{
			Self::Func(UnsafeItemSafetyOperator::Include(incpath)) =>
			{
				quote! { include_str!(#incpath) }
			},
			Self::String(s) =>
			{
				quote! { #s }
			}
		}
		.to_tokens(tokens);
	}
}

#[derive(Debug)]
pub(super) struct UnsafeItemAttrSafety
{
	kw:               kw::safety,
	parens:           Paren,
	pub(super) lines: Punctuated<UnsafeItemSafetyElem, Token![,]>
}

impl Parse for UnsafeItemAttrSafety
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		let content;
		Ok(Self {
			kw:     input.parse()?,
			parens: parenthesized!(content in input),
			lines:  safety_lines_from(&content)?
		})
	}
}

#[derive(Debug)]
pub(super) struct UnsafeItemAttrInto
{
	kw:     kw::into,
	parens: Paren,
	into:   UnsafeItemAttrIntoSpec
}

impl Parse for UnsafeItemAttrInto
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		let content;
		Ok(Self {
			kw:     input.parse()?,
			parens: parenthesized!(content in input),
			into:   content.parse()?
		})
	}
}

#[derive(Debug)]
enum UnsafeItemAttrIntoSpec
{
	/// `bare`
	Bare(kw::bare),

	/// `imp`
	Imp(kw::imp),
	/// `impl`
	Impl(Token![impl]),

	/// `interface`
	Interface(kw::interface),
	/// `trait`
	Trait(Token![trait]),

	/// `foreign`
	Foreign(kw::foreign),
	/// `extern`
	Extern(Token![extern])
}

impl Parse for UnsafeItemAttrIntoSpec
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		Ok(
			if input.peek(kw::bare)
			{
				Self::Bare(input.parse().expect_or_abort("unreachable code"))
			}
			else if input.peek(kw::imp)
			{
				Self::Imp(input.parse().expect_or_abort("unreachable code"))
			}
			else if input.peek(Token![impl])
			{
				Self::Impl(input.parse().expect_or_abort("unreachable code"))
			}
			else if input.peek(kw::interface)
			{
				Self::Interface(input.parse().expect_or_abort("unreachable code"))
			}
			else if input.peek(Token![trait])
			{
				Self::Trait(input.parse().expect_or_abort("unreachable code"))
			}
			else if input.peek(kw::foreign)
			{
				Self::Foreign(input.parse().expect_or_abort("unreachable code"))
			}
			else if input.peek(Token![extern])
			{
				Self::Extern(input.parse().expect_or_abort("unreachable code"))
			}
			else
			{
				Err(syn::Error::new(
					input.span(),
					"expected one of `bare`, `imp`, `impl`, `trait` or `interface`"
				))?
			}
		)
	}
}

#[derive(Debug, Default)]
pub(super) enum UnsafeItemKind
{
	#[default]
	Bare,
	InImpl,
	InTrait,
	InExtern
}

impl From<&UnsafeItemAttrIntoSpec> for UnsafeItemKind
{
	fn from(value: &UnsafeItemAttrIntoSpec) -> Self
	{
		match value
		{
			UnsafeItemAttrIntoSpec::Bare(_) => Self::Bare,
			UnsafeItemAttrIntoSpec::Imp(_) | UnsafeItemAttrIntoSpec::Impl(_) => Self::InImpl,
			UnsafeItemAttrIntoSpec::Interface(_) | UnsafeItemAttrIntoSpec::Trait(_) =>
			{
				Self::InTrait
			},
			UnsafeItemAttrIntoSpec::Foreign(_) | UnsafeItemAttrIntoSpec::Extern(_) => Self::InExtern
		}
	}
}

impl From<&UnsafeItemAttr> for UnsafeItemKind
{
	fn from(value: &UnsafeItemAttr) -> Self
	{
		match value
		{
			UnsafeItemAttr::SafetyOnly { .. } => Self::default(),
			UnsafeItemAttr::Detailled { into, .. } =>
			{
				into.as_ref()
					.map_or_default(|attr_into| (&attr_into.into).into())
			},
		}
	}
}

fn safety_lines_from(input: ParseStream)
-> syn::Result<Punctuated<UnsafeItemSafetyElem, Token![,]>>
{
	Ok(Punctuated::parse_terminated(input).and_then(|punctuated| {
		if punctuated.is_empty()
		{
			Err(syn::Error::new(
				punctuated.joined_span(),
				"you need to provide a valid justification for `unsafe`"
			))
		}
		else
		{
			Ok(punctuated)
		}
	})?)
}

fn eat_parens(input: ParseStream) -> syn::Result<ParseBuffer>
{
	let content;
	let _parens = parenthesized!(content in input);
	Ok(content)
}

fn parse_unsafe_check<T: Parse>(input: ParseStream) -> syn::Result<TokenStream2>
{
	let _ = input.parse::<T>().unwrap_or_abort();
	Ok(eat_parens(input)?.parse()?)
}
