#![allow(non_snake_case)]
#![feature(result_option_map_or_default)]
#![feature(iterator_try_collect)]
#![feature(proc_macro_span)]

use eager2::eager_proc_macro;
use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;

mod parse;

#[eager_proc_macro]
#[proc_macro_error]
pub fn constructors(input: TokenStream) -> TokenStream
{
	crate::parse::constructors(input.into()).into()
}
