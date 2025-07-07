#![allow(non_snake_case)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::{
	Expr,
	Ident,
	LitStr,
	Token,
	parse::{Parse, ParseStream},
	parse_macro_input
};

struct StaticAssertArgs
{
	condition: Expr,
	msg:       LitStr
}

fn make_random_ident<'a, 'b>(
	prefix: Option<&'a str>,
	suffix: Option<&'b str>
) -> Result<Ident, getrandom::Error>
{
	let rnd = getrandom::u64()?;
	let ident = format_ident!("{}{}{}", prefix.unwrap_or(""), rnd, suffix.unwrap_or(""));
	Ok(ident)
}

impl Parse for StaticAssertArgs
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		let cond: Expr = input.parse()?;
		if let Ok(_) = input.parse::<Token![,]>()
		{
			Ok(Self {
				condition: cond,
				msg:       input.parse()?
			})
		}
		else
		{
			Ok(Self {
				msg:       {
					let failed_string = format!(
						"static assertion `{}` failed",
						cond.clone().to_token_stream().to_string()
					);
					syn::parse2(quote! { #failed_string }).expect("internal error !")
				},
				condition: cond
			})
		}
	}
}

fn make_static_assert<T, U>(expr: T, msg: Option<U>) -> Result<TokenStream2, getrandom::Error>
where
	T: ToTokens,
	U: ToTokens
{
	let modname = make_random_ident(Some("__random_module_for_static_assert_"), Some("__"))?;
	let fnname = make_random_ident(Some("__random_fn_for_static_assert_"), Some("__"))?;
	Ok(quote! {
		mod #modname
		{
			use super::*;
			#[allow(dead_code)]
			const fn #fnname() {
				let condition: bool = (#expr);
				assert!(condition, #msg);
			}

			const _: () = #fnname();
		}
	})
}

impl ToTokens for StaticAssertArgs
{
	fn to_tokens(&self, tokens: &mut TokenStream2)
	{
		make_static_assert(&self.condition, Some(&self.msg))
			.unwrap_or_else(|err| {
				let errmsg = format!("unable to get random number: {}", err.to_string());
				quote! {
					compile_error!(#errmsg)
				}
			})
			.to_tokens(tokens);
	}
}

#[proc_macro]
pub fn static_assert(input: TokenStream) -> TokenStream
{
	let parsed = parse_macro_input!(input as StaticAssertArgs);

	quote! { #parsed }.into()
}
