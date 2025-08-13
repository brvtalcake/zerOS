use proc_macro2::TokenStream as TokenStream2;

pub(crate) fn constructors(input: TokenStream2) -> TokenStream2
{
    ::zerOS_constructors_macro::constructors(input.into()).into()
}
