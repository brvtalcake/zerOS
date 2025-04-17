//!
//! # proc-macro-utils
//!
//! This crate defines some utility proc_macro's that are
//! mainly used in the zerOS kernel.
//! For example, it defines a `bitfield!` macro, that can be
//! used like so:
//! ```rust
//! bitfield! {
//!     pub struct GDTDescriptor -> 64
//!     {
//!         pub u16 base_low: 12;
//!         u16 blah: 9;
//!         ...
//!         union
//!         {
//!             pub u8 access: 8;
//!             struct
//!             {
//!                 // detailed fields
//!             };
//!         };
//!     }
//! }
//! ```
//!

#![feature(proc_macro_expand)]
#![feature(proc_macro_totokens)]

extern crate proc_macro;

use std::mem::MaybeUninit;

use proc_macro::TokenStream as TokenStreamClassic;
use proc_macro2::Span;
#[allow(unused_imports)]
use proc_macro2::{Delimiter, Group, TokenStream};
use quote::{ToTokens, TokenStreamExt, format_ident, quote};
use syn::{
    Expr, Ident, Path, Token, Type, TypePath, braced,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str,
    punctuated::Punctuated,
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
        }
        Err(err) => TokenStream::from(err.to_compile_error()),
    };
    parsed.into()
}

#[proc_macro]
pub fn ctor(input: TokenStreamClassic) -> TokenStreamClassic
{
    if !input.is_empty()
    {
        let real_input: TokenStreamClassic = Group::new(Delimiter::Brace, input.into())
            .into_token_stream()
            .into();
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

mod kw
{
    use syn::custom_keyword;

    custom_keyword!(bitfield);
    custom_keyword!(bits);
}

trait AsTokenStream
{
    fn as_token_stream(&self, struct_name: &Ident, range_base: &Expr) -> TokenStream;
}

#[derive(Debug)]
struct BitFieldElemOnly
{
    vis: Option<Token![pub]>,
    ty: Type,
    ident: Ident,
    sep: Token![:],
    size: Expr,
}

impl Parse for BitFieldElemOnly
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        Ok(Self {
            vis: input.parse().unwrap_or(None),
            ty: input.parse()?,
            ident: input.parse()?,
            sep: input.parse()?,
            size: input.parse()?,
        })
    }
}

impl AsTokenStream for BitFieldElemOnly
{
    fn as_token_stream(&self, struct_name: &Ident, range_base: &Expr) -> TokenStream
    {
        let ident = &self.ident;
        let retty = &self.ty;
        let vi = if let Some(visibility) = self.vis
        {
            visibility.into_token_stream()
        }
        else
        {
            TokenStream::new()
        };
        let sz = &self.size;
        quote! {
            #[overloadf::overload]
            impl #struct_name
            {
                #vi fn #ident(&self) -> #retty
                {
                    let arr = &self.0;
                    let ret: #retty = 0;
                    let counter: usize = 0;
                    const START: usize = #range_base;
                    const END  : usize = #range_base + #sz;
                    for i in START..END
                    {
                        let tmp = ((arr[i / 8] >> (i % 8)) & 1) as #retty;
                        ret |= tmp << counter;
                        counter += 1;
                    }
                    ret
                }

                #vi fn #ident(&mut self, val: #retty)
                {
                    let arr = &mut self.0;
                    let counter: usize = 0;
                    const START: usize = #range_base;
                    const END  : usize = #range_base + #sz;
                    for i in START..END
                    {
                        let bit = ((val >> counter) & 1) as bool;
                        counter += 1;
                        arr[i / 8] = (arr[i / 8] & !((1 as u8) << (i % 8))) | ((bit as u8) << (i % 8));
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
enum BitFieldElemInner
{
    ElemOnly(BitFieldElemOnly),
    ElemStruct(BitFieldElemInnerStruct),
}

impl BitFieldElemInner
{
    fn size(&self) -> Expr
    {
        match self
        {
            Self::ElemOnly(elem) => elem.size.clone(),
            Self::ElemStruct(elem) =>
            {
                let mut elems = vec![];
                for el in &elem.elems
                {
                    elems.push(&el.size);
                }
                let tokstream = elems
                    .iter()
                    .skip(1)
                    .fold(elems[0].to_token_stream(), |acc, el| {
                        quote! {
                            (#acc) + (#el)
                        }
                    });
                syn::parse2(tokstream).unwrap()
            }
        }
    }
}

impl AsTokenStream for BitFieldElemInner
{
    fn as_token_stream(&self, struct_name: &Ident, range_base: &Expr) -> TokenStream
    {
        match self
        {
            Self::ElemOnly(elem) => elem.as_token_stream(struct_name, range_base),
            Self::ElemStruct(elem) => elem.as_token_stream(struct_name, range_base),
        }
    }
}

impl Parse for BitFieldElemInner
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        let mut errors: MaybeUninit<syn::Error> = MaybeUninit::uninit();
        if let Ok(elem) = input.parse::<BitFieldElemOnly>().or_else(|err| {
            errors.write(err.clone());
            Err(err)
        })
        {
            return Ok(Self::ElemOnly(elem));
        }
        let mut errors = unsafe { errors.assume_init() };
        if let Ok(elem) = input.parse::<BitFieldElemInnerStruct>().or_else(|err| {
            errors.combine(err.clone());
            Err(err)
        })
        {
            return Ok(Self::ElemStruct(elem));
        }
        Err(errors)
    }
}

#[derive(Debug)]
struct BitFieldElemInnerStruct
{
    kw: Token![struct],
    braces: syn::token::Brace,
    elems: Punctuated<BitFieldElemOnly, Token![;]>,
}

impl AsTokenStream for BitFieldElemInnerStruct
{
    fn as_token_stream(&self, struct_name: &Ident, range_base: &Expr) -> TokenStream
    {
        let mut stream: TokenStream = TokenStream::new();
        let mut new_range_base = range_base.clone();
        for elem in &self.elems
        {
            stream.extend(elem.as_token_stream(struct_name, &new_range_base));

            let elem_size = &elem.size;
            let new_size = quote! { (#new_range_base) + (#elem_size) };

            new_range_base = match syn::parse2(new_size)
            {
                Ok(sz) => sz,
                Err(_) => unreachable!(),
            };
        }
        stream
    }
}

impl Parse for BitFieldElemInnerStruct
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        let content;
        Ok(Self {
            kw: input.parse()?,
            braces: braced!(content in input),
            elems: <Punctuated<_, _>>::parse_terminated(&content)?,
        })
    }
}

#[derive(Debug)]
struct BitFieldElemUnion
{
    kw: Token![union],
    braces: syn::token::Brace,
    elems: Punctuated<BitFieldElemInner, Token![;]>,
}

impl BitFieldElemUnion
{
    fn size(&self) -> Expr
    {
        let elems = self.elems.iter().map(|el| el.size());
        syn::parse2(quote! {
            (
                const {
                    let ___arr = [ #(#elems,)* ];
                    let ___res: usize = 0;
                    for ___el in ___arr
                    {
                        if ___el > ___res
                        {
                            ___res = ___el;
                        }
                    }
                    ___res
                }
            )
        })
        .unwrap()
    }
}

impl AsTokenStream for BitFieldElemUnion
{
    fn as_token_stream(&self, struct_name: &Ident, range_base: &Expr) -> TokenStream
    {
        let mut stream: TokenStream = TokenStream::new();
        for elem in &self.elems
        {
            stream.extend(elem.as_token_stream(struct_name, range_base));
        }
        stream
    }
}

impl Parse for BitFieldElemUnion
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
    {
        let content;
        Ok(Self {
            kw: input.parse()?,
            braces: braced!(content in input),
            elems: <Punctuated<_, _>>::parse_terminated(&content)?,
        })
    }
}

#[derive(Debug)]
enum BitFieldElem
{
    ElemOnly(BitFieldElemOnly),
    ElemUnion(BitFieldElemUnion),
}

impl BitFieldElem
{
    fn size(&self) -> Expr
    {
        match self
        {
            Self::ElemOnly(elem) => elem.size.clone(),
            Self::ElemUnion(elem) => elem.size(),
        }
    }
}

impl Parse for BitFieldElem
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
    {
        let mut errors: MaybeUninit<syn::Error> = MaybeUninit::uninit();
        if let Ok(elem) = input.fork().parse::<BitFieldElemOnly>().or_else(|err| {
            errors.write(err.clone());
            Err(err)
        })
        {
            let _ = input.parse::<BitFieldElemOnly>();
            return Ok(Self::ElemOnly(elem));
        }
        let mut errors = unsafe { errors.assume_init() };
        if let Ok(elem) = input.parse::<BitFieldElemUnion>().or_else(|err| {
            errors.combine(err.clone());
            Err(err)
        })
        {
            return Ok(Self::ElemUnion(elem));
        }
        Err(errors)
    }
}

impl AsTokenStream for BitFieldElem
{
    fn as_token_stream(&self, struct_name: &Ident, range_base: &Expr) -> TokenStream
    {
        match self
        {
            Self::ElemOnly(elem) => elem.as_token_stream(struct_name, range_base),
            Self::ElemUnion(elem) => elem.as_token_stream(struct_name, range_base),
        }
    }
}

#[derive(Debug)]
struct BitFieldBlockOuter
{
    braces: syn::token::Brace,
    elems: Punctuated<BitFieldElem, Token![;]>,
}
impl Parse for BitFieldBlockOuter
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
    {
        let content;
        Ok(Self {
            braces: braced!(content in input),
            elems: <Punctuated<BitFieldElem, Token![;]>>::parse_terminated(&content)?,
        })
    }
}

#[derive(Debug)]
struct BitFieldDecl
{
    attrs: Vec<syn::Attribute>,
    vis: Option<Token![pub]>,
    kw: Token![struct],
    name: syn::Ident,
    arrow: Token![->],
    underlying_size: syn::LitInt,
    block: BitFieldBlockOuter,
}

impl BitFieldDecl
{
    fn choose_underlying(&self) -> Type
    {
        // TODO: handle errors
        let size: u128 = self.underlying_size.base10_parse().unwrap();
        let typestr = format!("[u8; {}]", (size / 8) + (if size % 8 != 0 { 1 } else { 0 }));
        parse_str(&typestr).unwrap()
    }
}

impl Parse for BitFieldDecl
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
    {
        Ok(Self {
            attrs: input.call(syn::Attribute::parse_outer)?,
            vis: input.parse().unwrap_or(None),
            kw: input.parse()?,
            name: input.parse()?,
            arrow: input.parse()?,
            underlying_size: input.parse()?,
            block: input.parse()?,
        })
    }
}

fn make_random_ident<'a, 'b>(
    prefix: Option<&'a str>,
    suffix: Option<&'b str>,
) -> Result<Ident, getrandom::Error>
{
    let rnd = getrandom::u64()?;
    let ident = format_ident!("{}{}{}", prefix.unwrap_or(""), rnd, suffix.unwrap_or(""));
    Ok(ident)
}

fn make_static_assert_lte(lhs: Expr, rhs: Expr) -> Result<TokenStream, getrandom::Error>
{
    let modname = make_random_ident(Some("__random_module_for_static_assert_"), Some("__"))?;
    let fnname = make_random_ident(Some("__random_fn_for_static_assert_"), Some("__"))?;
    Ok(quote! {
        mod #modname
        {
            #[allow(dead_code)]
            const fn #fnname() {
                assert!(#lhs <= #rhs);
            }

            const _: () = #fnname();
        }
    })
}

impl ToTokens for BitFieldDecl
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let attrs = &self.attrs;
        let vis = if let Some(visibility) = self.vis
        {
            visibility.into_token_stream()
        }
        else
        {
            TokenStream::new()
        };
        let name = &self.name;
        let underlying_type = self.choose_underlying();
        let struct_decl = quote! {
            #(#attrs)*
            #vis struct #name (#underlying_type);
        };
        struct_decl.to_tokens(tokens);
        let mut range_base: Expr = syn::parse2(quote! { 0 }).unwrap();
        for decl in &self.block.elems
        {
            let sz = decl.size();
            let tokstream = decl.as_token_stream(name, &range_base);
            tokens.extend(tokstream);
            range_base = syn::parse2(quote! {
                (#range_base) + (#sz)
            })
            .unwrap();
        }
        //let type_to_check = quote! {
        //    [u8; (#range_base / 8) + (if #range_base % 8 { 1 } else { 0 })]
        //};
        //tokens.extend(
        //    quote! {
        //        static_assertions::assert_eq_size!(
        //            #type_to_check, #underlying_type
        //        );
        //    }
        //);
        let underlying_sz = &self.underlying_size;
        tokens.extend(
            make_static_assert_lte(range_base, syn::parse2(quote! { #underlying_sz }).unwrap())
                .unwrap_or_else(|err| {
                    let errstr = format!("{err}\n");
                    quote! { compile_error!(#errstr) }
                }),
        );
    }
}

#[proc_macro]
pub fn bitfield(input: TokenStreamClassic) -> TokenStreamClassic
{
    let parsed = parse_macro_input!(input as BitFieldDecl);

    parsed.into_token_stream().into()
}

#[cfg(test)]
mod tests
{
    use syn::parse_str;

    use crate::BitFieldElemUnion;

    #[test]
    fn union_test()
    {
        let to_parse = r#"
            union {
                pub u8 test1: 4;
                pub u8 test2: 4;
            };"#;
        let union_: BitFieldElemUnion = parse_str(to_parse).unwrap_or_else(|err| {
            println!("{}", err.clone());
            err
        })?;
    }
}
