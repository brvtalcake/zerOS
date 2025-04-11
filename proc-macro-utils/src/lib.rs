#![feature(proc_macro_expand)]
#![feature(proc_macro_totokens)]

extern crate proc_macro;

use proc_macro::{ToTokens, TokenStream};

#[proc_macro]
pub fn array_size(tt: TokenStream) -> TokenStream
{
    let expanded_tt = tt.expand_expr().unwrap_or(tt);
    let parsed = match syn::parse::<syn::Expr>(expanded_tt)
    {
        Ok(syn::Expr::Array(parsed_array)) => {
            parsed_array.elems.len().to_token_stream()
        },
        Ok(expr) => {
            let err_string: String = "this procedural macro only accepts array expressions".into();
            let formatted_expr = format!("{:?}", expr);
            syn::Error::new_spanned(
                expr, err_string + &formatted_expr
            ).to_compile_error().into()
        },
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error())
    };
    parsed
}