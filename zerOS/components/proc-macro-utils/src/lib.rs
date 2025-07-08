#![allow(non_snake_case)]
#![feature(proc_macro_expand)]

use proc_macro::TokenStream as TokenStreamClassic;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
	Expr,
	TypeArray,
	parse::{Parse, ParseStream},
	parse_macro_input
};

#[proc_macro]
pub fn array_size(tt: TokenStreamClassic) -> TokenStreamClassic
{
	let expanded_tt = tt.expand_expr().unwrap_or(tt);
	let parsed = match syn::parse::<syn::Expr>(expanded_tt)
	{
		Ok(syn::Expr::Array(parsed_array)) => ToTokens::to_token_stream(&parsed_array.elems.len()),
		Ok(expr) =>
		{
			let err_string: String = "this procedural macro only accepts array expressions".into();
			let formatted_expr = format!("{:?}", expr);
			syn::Error::new_spanned(expr, err_string + &formatted_expr)
				.to_compile_error()
				.into()
		},
		Err(err) => TokenStream::from(err.to_compile_error())
	};
	parsed.into()
}

#[proc_macro]
pub fn random_number(_input: TokenStreamClassic) -> TokenStreamClassic
{
	if let Ok(rand) = getrandom::u32().map(|num| syn::Index::from(num as usize))
	{
		quote! { #rand }
	}
	else
	{
		quote! {
			compile_error!("random_number: unable to get random number");
		}
	}
	.into()
}

#[proc_macro]
pub fn random_ident(_input: TokenStreamClassic) -> TokenStreamClassic
{
	if let Ok(rand) = getrandom::u32().map(|num| format_ident!("randomly_generated_ident_{}", num))
	{
		quote! { #rand }
	}
	else
	{
		quote! {
			compile_error!("random_ident: unable to get random number");
		}
	}
	.into()
}

mod kw
{
	use syn::custom_keyword;

	custom_keyword!(with);
}

#[allow(dead_code)]
struct ConstInitArrayArgs
{
	arrty: TypeArray,
	with:  kw::with,
	expr:  Expr
}

impl Parse for ConstInitArrayArgs
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		Ok(Self {
			arrty: input.parse()?,
			with:  input.parse()?,
			expr:  input.parse()?
		})
	}
}

impl ToTokens for ConstInitArrayArgs
{
	fn to_tokens(&self, tokens: &mut TokenStream)
	{
		let array_size = &self.arrty.len;
		let array_type = self.arrty.elem.as_ref();
		let parsed_expr = &self.expr;
		tokens.extend(quote! {
			const {
				const fn ___init_it___() -> [#array_type; #array_size]
				{
					let mut ret: [::core::mem::MaybeUninit<#array_type>; #array_size] = [
						const {
							::core::mem::MaybeUninit::uninit()
						}; #array_size
					];
					let mut i = (#array_size - 1) as isize;
					while i >= 0
					{
						ret[i as usize].write(#parsed_expr);
						i -= 1;
					}
					unsafe { ::core::mem::transmute(ret) }
				}
				___init_it___()
			}
		});
	}
}

/// # Example
/// ```rust
/// use std::sync::atomic::AtomicBool;
///
/// #[macro_export]
/// extern crate proc_macro_utils;
/// use proc_macro_utils::constinit_array_with;
///
/// static ATOMIC_ARRAY: [AtomicBool; 42] =
/// 	constinit_array!([AtomicBool; 42] with AtomicBool::new(false));
/// # fn main() {}
/// ```
#[proc_macro]
pub fn constinit_array(input: TokenStreamClassic) -> TokenStreamClassic
{
	let parsed = parse_macro_input!(input as ConstInitArrayArgs);

	quote! { #parsed }.into()
}

//#[proc_macro_attribute]
// pub fn embpp(input: TokenStreamClassic) -> TokenStreamClassic
//{
// 	todo!()
//}
