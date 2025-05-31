use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{LitInt, Stmt, Token, parenthesized, parse::Parse};

use crate::punct;

mod kw
{
	use syn::custom_keyword;

	custom_keyword!(priority);
}

struct CtorPrio
{
	prio: usize
}

impl Parse for CtorPrio
{
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
	{
		// return Err(syn::Error::new(Span::call_site(),
		// input.span().source_text().unwrap()));
		// let _at = input.parse::<punct::AtSign>()?;
		// let _prio_kw = input.parse::<kw::priority>()?;
		// let prio;
		// let _parens = parenthesized!(prio in input);
		// let _semicolon = input.parse::<Token![;]>()?;
		// Ok(Self {
		// 	prio: prio.parse::<LitInt>()?.base10_parse()?
		//})
		let _prio_kw = input.parse::<kw::priority>()?;
		let prio = input.parse::<LitInt>()?.base10_parse()?;
		let _semicolon = input.parse::<Token![;]>()?;
        println!("WE REACHED THIS POINT");
		Ok(Self { prio })
	}
}

impl ToTokens for CtorPrio
{
	fn to_tokens(&self, tokens: &mut TokenStream)
	{
		let secname = format!(".ctors_init_array.{}", self.prio);
		tokens.extend(quote! {
			#[unsafe(link_section = #secname)]
		});
	}
}

pub(crate) struct CtorInput
{
	prio: CtorPrio,
	body: Vec<Stmt>
}

impl Parse for CtorInput
{
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
	{
		Ok(Self {
			prio: input.parse()?,
			body: {
				let mut ret = vec![];
                let rest = input.cursor().token_stream().to_string();
                println!("{rest}");
				while !input.is_empty()
				{
					ret.push(input.parse()?);
				}
				ret
			}
		})
	}
}

impl ToTokens for CtorInput
{
	fn to_tokens(&self, tokens: &mut TokenStream)
	{
		let section = self.prio.to_token_stream();
		let body = &self.body;
		let extend_with = if let Ok(rnd) = getrandom::u64()
		{
			let modident = format_ident!("__local_ctors__{}", rnd);
			let exported_fn = format_ident!("__local_ctors_fn_exported__{}", rnd);
			let exported_ctor_ptr = format_ident!(
				"{}{}",
				"__local_ctors_ptr_exported__".to_ascii_uppercase(),
				rnd
			);
			quote! {
				mod #modident
				{
					use super::*;

					#[unsafe(no_mangle)]
					#[unsafe(link_section = ".bootcode")]
					unsafe extern "C" fn #exported_fn ()
					{
						#(#body)*
					}

					#section
					#[used(linker)]
					#[allow(non_upper_case_globals)]
					static #exported_ctor_ptr: crate::init::ctors::Ctor = #exported_fn;
				}
			}
		}
		else
		{
			quote! { compile_error!("unable to get random number") }
		};
		tokens.extend(extend_with);
	}
}
