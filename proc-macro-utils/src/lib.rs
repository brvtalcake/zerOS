#![feature(proc_macro_expand)]
#![feature(proc_macro_totokens)]

extern crate proc_macro;

use proc_macro::{Delimiter, ToTokens, TokenStream as TokenStreamClassic};
#[allow(unused_imports)]
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

#[proc_macro]
pub fn array_size(tt: TokenStreamClassic) -> TokenStreamClassic
{
    let expanded_tt = tt.expand_expr().unwrap_or(tt);
    let parsed = match syn::parse::<syn::Expr>(expanded_tt)
    {
        Ok(syn::Expr::Array(parsed_array)) => parsed_array.elems.len().to_token_stream(),
        Ok(expr) =>
        {
            let err_string: String = "this procedural macro only accepts array expressions".into();
            let formatted_expr = format!("{:?}", expr);
            syn::Error::new_spanned(expr, err_string + &formatted_expr)
                .to_compile_error()
                .into()
        }
        Err(err) => TokenStreamClassic::from(err.to_compile_error()),
    };
    parsed
}

#[proc_macro]
pub fn ctor(input: TokenStreamClassic) -> TokenStreamClassic
{
    if !input.is_empty()
    {
        let real_input = proc_macro::Group::new(Delimiter::Brace, input).into_token_stream();
        let body = parse_macro_input!(real_input as syn::Block);
        if let Ok(rnd) = getrandom::u64()
        {
            let modident = format_ident!("__local_ctors__{}", rnd);
            let exported_fn = format_ident!("__local_ctors_fn_exported__{}", rnd);
            quote! {
                mod #modident
                {
                    #[unsafe(no_mangle)]
                    #[unsafe(link_section = ".ctors_init_array")]
                    unsafe extern "C" fn #exported_fn ()
                    #body
                }
            }
            .into()
        }
        else
        {
            quote! { compile_error!("unable to get random number") }.into()
        }
    }
    else
    {
        quote! {}.into()
    }
}
