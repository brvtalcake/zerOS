use proc_macro_error2::{ResultExt as _, emit_warning};
use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{
	Block,
	Expr,
	Meta,
	MetaList,
	MetaNameValue,
	Stmt,
	parse::{Parse, ParseStream},
	spanned::Spanned
};

use crate::imp::{GetAttrs as _, IdentCapable, InnerSafetyDocAttr};

struct Statement
{
	pre:  Option<Expr>,
	post: Option<Expr>,
	stmt: Stmt
}

fn make_cond_check(cond: &Expr, prefix: Option<&str>, span: Option<Span2>) -> TokenStream2
{
	let prefix = prefix.unwrap_or_default();
	if let Some(span) = span
	{
		quote_spanned! {
			span => {
				debug_assert!(
					#cond,
                    concat!(
						"unsafe ",
						#prefix,
						"condition check failed: ",
						stringify!(#cond)
					)
				)
			}
		}
	}
	else
	{
		quote! {
			{
				debug_assert!(
                    #cond,
					concat!(
						"unsafe ",
						#prefix,
						"condition check failed: ",
						stringify!(#cond)
					)
				)
			}
		}
	}
}

impl Statement
{
	fn expand(&self, safety: &InnerSafetyDocAttr) -> TokenStream2
	{
		let span = self.stmt.span();
		let pre = self
			.pre
			.as_ref()
			.map(|cond| make_cond_check(cond, Some("pre"), Some(span)));
		let post = self
			.post
			.as_ref()
			.map(|cond| make_cond_check(cond, Some("post"), Some(span)));
		let ident = format_ident!(
			"__expanded_by_zeros_unsafe_at_file{}_line{}_column{}",
			span.unwrap().file().as_ident_name(),
			span.unwrap().line(),
			span.unwrap().column()
		);
		let safety = &safety.safety;
		match &self.stmt
		{
			Stmt::Expr(expr, semi) =>
			{
				quote_spanned! {
					span =>
						#pre
						#(#[doc = #safety])*
						let #ident = #expr;
						#post
						#ident #semi
				}
			},
			Stmt::Item(item) =>
			{
				quote_spanned! {
					span =>
						#pre
						#(#[doc = #safety])*
						#item
						#post
				}
			},
			Stmt::Local(local) =>
			{
				quote_spanned! {
					span =>
						#pre
						#(#[doc = #safety])*
						#local
						#post
				}
			},
			Stmt::Macro(mac) =>
			{
				quote_spanned! {
					span =>
						#pre
						#(#[doc = #safety])*
						#mac
						#post
				}
			}
		}
	}
}

impl Parse for Statement
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		input.parse::<Stmt>().and_then(TryInto::try_into)
	}
}

impl TryFrom<Stmt> for Statement
{
	type Error = syn::Error;

	fn try_from(mut stmt: Stmt) -> syn::Result<Self>
	{
		let mut pre = None;
		let mut post = None;
		if let Some(attrs) = stmt.attrs_mut()
		{
			attrs
				.extract_if(.., |attribute| {
					let ident = attribute.path().require_ident().unwrap_or_abort();
					match ident.to_string().as_str()
					{
						"pre" | "precond" | "precondition" => Some(&mut pre),
						"post" | "postcond" | "postcondition" => Some(&mut post),
						_ => None
					}
					.map(|reference| {
						*reference = match &attribute.meta
						{
							Meta::NameValue(MetaNameValue { value, .. }) => Some(value.clone()),
							Meta::List(MetaList { tokens, .. }) =>
							{
								Some(syn::parse2(tokens.clone()).unwrap_or_abort())
							},
							Meta::Path(_) =>
							{
								emit_warning!(
									ident, "expected an expression after `{}`", ident;
									note = "ignoring the {}",
									if ident.to_string().starts_with("pr") {
										"precondition"
									} else {
										"postcondition"
									};
								);
								None
							}
						};
						true
					})
					.unwrap_or(false)
				})
				.for_each(drop);
		}
		Ok(Self { pre, post, stmt })
	}
}

struct UnsafeBlock
{
	safety: InnerSafetyDocAttr,
	stmts:  Vec<Statement>
}

impl Parse for UnsafeBlock
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		let safety = input.parse()?;
		let stmts = input
			.call(Block::parse_within)?
			.into_iter()
			.map(TryInto::try_into)
			.try_collect()?;
		Ok(Self { safety, stmts })
	}
}

impl ToTokens for UnsafeBlock
{
	fn to_tokens(&self, tokens: &mut TokenStream2)
	{
		let safety = &self.safety.safety;
		let inner_attrs = &self.safety.others;
		let stmts = self.stmts.iter().map(|stmt| stmt.expand(&self.safety));
		quote! {
			#[expect(unsafe_code)]
			#[allow(unused_doc_comments)]
			#(#[doc = #safety])*
			unsafe {
				#(#inner_attrs)*
				#(#stmts)*
			}
		}
		.to_tokens(tokens);
	}
}

pub(crate) fn block(input: TokenStream2) -> TokenStream2
{
	syn::parse2::<UnsafeBlock>(input)
		.unwrap_or_abort()
		.into_token_stream()
}
