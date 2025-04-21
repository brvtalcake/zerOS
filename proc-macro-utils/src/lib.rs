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

use proc_macro::TokenStream as TokenStreamClassic;
use proc_macro2::{Delimiter, Group, Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use std::mem::MaybeUninit;
use syn::{
    Attribute, Expr, ExprAssign, Ident, Lit, LitStr, Token, Type, braced,
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

#[allow(dead_code)]
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
        /* quote! {
            #[overloadf::overload]
            impl #struct_name
            {
                #vi const fn #ident(&self) -> #retty
                {
                    let arr = &self.0;
                    let mut ret: #retty = 0;
                    let mut counter: usize = 0;
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

                #vi const fn #ident(&mut self, val: #retty)
                {
                    let arr = &mut self.0;
                    let mut counter: usize = 0;
                    const START: usize = #range_base;
                    const END  : usize = #range_base + #sz;
                    for i in START..END
                    {
                        let bit: bool = ((val >> counter) & 1) != 0;
                        counter += 1;
                        arr[i / 8] = (arr[i / 8] & !((1 as u8) << (i % 8))) | ((bit as u8) << (i % 8));
                    }
                }
            }
        } */
       let getfn  = format_ident!("get_{}", ident);
       let setfn  = format_ident!("set_{}", ident);
       let withfn = format_ident!("with_{}", ident);
        quote! {
            
            impl #struct_name
            {
                #vi fn #getfn(&self) -> #retty
                {
                    let arr = &self.0;
                    let mut ret: #retty = 0;
                    let mut counter: usize = 0;
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

                #vi fn #setfn(&mut self, val: #retty)
                {
                    let arr = &mut self.0;
                    let mut counter: usize = 0;
                    const START: usize = #range_base;
                    const END  : usize = #range_base + #sz;
                    for i in START..END
                    {
                        let bit: bool = ((val >> counter) & 1) != 0;
                        counter += 1;
                        arr[i / 8] = (arr[i / 8] & !((1 as u8) << (i % 8))) | ((bit as u8) << (i % 8));
                    }
                }

                #vi fn #withfn(&mut self, val: #retty) -> &mut Self
                {
                    self.#setfn(val);
                    self
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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
        let elems = self.elems.iter().skip(1).map(|el| el.size());
        let first = self.elems[0].size();
        syn::parse2(quote! {
            (
                static_max!((#first as usize) #(,(#elems) as usize)*)
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

fn make_static_assert_cmp<T, U>(
    lhs: T,
    rhs: U,
    cmpsym: TokenStream,
) -> Result<TokenStream, getrandom::Error>
where
    T: ToTokens,
    U: ToTokens,
{
    let modname = make_random_ident(Some("__random_module_for_static_assert_"), Some("__"))?;
    let fnname = make_random_ident(Some("__random_fn_for_static_assert_"), Some("__"))?;
    Ok(quote! {
        mod #modname
        {
            #[allow(dead_code)]
            const fn #fnname() {
                const CONDITION: bool = ((#lhs) #cmpsym (#rhs));
                assert!(CONDITION);
            }

            const _: () = #fnname();
        }
    })
}

#[derive(Debug)]
enum BitFieldRequest
{
    Constructor
    {
        impl_it: bool
    },
    Default
    {
        impl_it: bool
    },
    AsRef
    {
        impl_it: bool
    },
    AsMut
    {
        impl_it: bool
    },
    SizeEq
    {
        cmp_with: usize
    },
    SizeNeq
    {
        cmp_with: usize
    },
    SizeGt
    {
        cmp_with: usize
    },
    SizeLt
    {
        cmp_with: usize
    },
    SizeGte
    {
        cmp_with: usize
    },
    SizeLte
    {
        cmp_with: usize
    },
}

impl BitFieldRequest
{
    fn expand(&self, struct_name: &Ident, summed_size: &Expr, underlying_type: &Type)
    -> TokenStream
    {
        match self
        {
            Self::Constructor { impl_it } =>
            {
                if *impl_it
                {
                    impl_new_for(struct_name)
                }
                else
                {
                    TokenStream::new()
                }
            }
            Self::Default { impl_it } =>
            {
                if *impl_it
                {
                    impl_default_for(struct_name)
                }
                else
                {
                    TokenStream::new()
                }
            }
            Self::AsRef { impl_it } =>
            {
                if *impl_it
                {
                    impl_as_ref_for(struct_name, underlying_type)
                }
                else
                {
                    TokenStream::new()
                }
            }
            Self::AsMut { impl_it } =>
            {
                if *impl_it
                {
                    impl_as_mut_for(struct_name, underlying_type)
                }
                else
                {
                    TokenStream::new()
                }
            }
            Self::SizeEq { cmp_with } =>
            {
                make_static_assert_cmp(summed_size, cmp_with, quote! { == }).unwrap()
            }
            Self::SizeNeq { cmp_with } =>
            {
                make_static_assert_cmp(summed_size, cmp_with, quote! { != }).unwrap()
            }
            Self::SizeGt { cmp_with } =>
            {
                make_static_assert_cmp(summed_size, cmp_with, quote! { > }).unwrap()
            }
            Self::SizeGte { cmp_with } =>
            {
                make_static_assert_cmp(summed_size, cmp_with, quote! { >= }).unwrap()
            }
            Self::SizeLt { cmp_with } =>
            {
                make_static_assert_cmp(summed_size, cmp_with, quote! { < }).unwrap()
            }
            Self::SizeLte { cmp_with } =>
            {
                make_static_assert_cmp(summed_size, cmp_with, quote! { <= }).unwrap()
            }
        }
    }
}

fn parse_yesno(lit: &Lit) -> Result<bool, syn::Error>
{
    match lit
    {
        Lit::Bool(val) => Ok(val.value()),
        Lit::Byte(val) => Ok(val.value() != b'0'),
        Lit::ByteStr(val) => Ok({
            let ascii = val.value().to_ascii_uppercase();
            ascii == b"YES" || ascii == b"ALWAYS" || ascii == b"TRUE" || ascii == b"ON"
        }),
        Lit::CStr(val) => Ok({
            let ascii = val.value().to_bytes().to_ascii_uppercase();
            ascii == b"YES" || ascii == b"ALWAYS" || ascii == b"TRUE" || ascii == b"ON"
        }),
        Lit::Char(val) => Ok(val.value() != '0'),
        Lit::Float(val) => Ok({
            let float: f64 = val.base10_parse()?;
            float.round() != 0.0
        }),
        Lit::Int(val) => Ok({
            let integer: i128 = val.base10_parse()?;
            integer != 0
        }),
        Lit::Str(val) => Ok(val.value() == "YES"
            || val.value() == "ALWAYS"
            || val.value() == "TRUE"
            || val.value() == "ON"),
        _ => Err(syn::Error::new(Span::call_site(), "invalid literal")),
    }
}

impl BitFieldDecl
{
    fn handle_attrs(&self) -> (Vec<BitFieldRequest>, Vec<Attribute>)
    {
        let mut reqs: Vec<BitFieldRequest> = vec![];
        let mut others: Vec<Attribute> = vec![];
        for attr in &self.attrs
        {
            if let Some(ident) = attr.path().get_ident()
            {
                if ident.to_string().to_uppercase() == "PROVIDE"
                {
                    if let Some(ExprAssign {
                        attrs: _,
                        left: lhs,
                        eq_token: _,
                        right: rhs,
                    }) = attr.parse_args().ok()
                    {
                        if let Expr::Path(lpath) = lhs.as_ref()
                        {
                            if let Expr::Lit(rlit) = rhs.as_ref()
                            {
                                if let Ok(as_str) =
                                    lpath.path.require_ident().map(|ok| ok.to_string())
                                {
                                    match as_str.to_uppercase().as_str()
                                    {
                                        "CONSTRUCTOR" | "NEW" | "CTOR" =>
                                        {
                                            if let Ok(yesno) = parse_yesno(&rlit.lit)
                                            {
                                                if let Some(elem) =
                                                    reqs.iter_mut().find(|el| match el
                                                    {
                                                        BitFieldRequest::Constructor {
                                                            impl_it: _,
                                                        } => true,
                                                        _ => false,
                                                    })
                                                {
                                                    *elem = BitFieldRequest::Constructor {
                                                        impl_it: yesno,
                                                    };
                                                    continue;
                                                }
                                                else
                                                {
                                                    reqs.push(BitFieldRequest::Constructor {
                                                        impl_it: yesno,
                                                    });
                                                    continue;
                                                }
                                            }
                                        }
                                        "DEFAULT" =>
                                        {
                                            if let Ok(yesno) = parse_yesno(&rlit.lit)
                                            {
                                                if let Some(elem) =
                                                    reqs.iter_mut().find(|el| match el
                                                    {
                                                        BitFieldRequest::Default { impl_it: _ } =>
                                                        {
                                                            true
                                                        }
                                                        _ => false,
                                                    })
                                                {
                                                    *elem =
                                                        BitFieldRequest::Default { impl_it: yesno };
                                                    continue;
                                                }
                                                else
                                                {
                                                    reqs.push(BitFieldRequest::Default {
                                                        impl_it: yesno,
                                                    });
                                                    continue;
                                                }
                                            }
                                        }
                                        "ASREF" | "AS_REF" =>
                                        {
                                            if let Ok(yesno) = parse_yesno(&rlit.lit)
                                            {
                                                if let Some(elem) =
                                                    reqs.iter_mut().find(|el| match el
                                                    {
                                                        BitFieldRequest::AsRef { impl_it: _ } =>
                                                        {
                                                            true
                                                        }
                                                        _ => false,
                                                    })
                                                {
                                                    *elem =
                                                        BitFieldRequest::AsRef { impl_it: yesno };
                                                    continue;
                                                }
                                                else
                                                {
                                                    reqs.push(BitFieldRequest::AsRef {
                                                        impl_it: yesno,
                                                    });
                                                    continue;
                                                }
                                            }
                                        }
                                        "ASMUT" | "AS_MUT" =>
                                        {
                                            if let Ok(yesno) = parse_yesno(&rlit.lit)
                                            {
                                                if let Some(elem) =
                                                    reqs.iter_mut().find(|el| match el
                                                    {
                                                        BitFieldRequest::AsMut { impl_it: _ } =>
                                                        {
                                                            true
                                                        }
                                                        _ => false,
                                                    })
                                                {
                                                    *elem =
                                                        BitFieldRequest::AsMut { impl_it: yesno };
                                                    continue;
                                                }
                                                else
                                                {
                                                    reqs.push(BitFieldRequest::AsMut {
                                                        impl_it: yesno,
                                                    });
                                                    continue;
                                                }
                                            }
                                        }
                                        _ =>
                                        {}
                                    }
                                }
                            }
                        }
                    }
                }
                else if ident.to_string().to_uppercase() == "CHECK"
                {
                    if let Some(ExprAssign {
                        attrs: _,
                        left: lhs,
                        eq_token: _,
                        right: rhs,
                    }) = attr.parse_args().ok()
                    {
                        match (lhs.as_ref(), rhs.as_ref())
                        {
                            (Expr::Path(ident), Expr::Lit(lit)) =>
                            {
                                match (
                                    ident
                                        .path
                                        .require_ident()
                                        .map(|okval| okval.to_string().to_uppercase()),
                                    &lit.lit,
                                )
                                {
                                    (Ok(val), Lit::Int(intlit))
                                        if val == "EQUAL_TO".to_string()
                                            || val == "EQ".to_string() =>
                                    {
                                        reqs.push(BitFieldRequest::SizeEq {
                                            cmp_with: intlit.base10_parse().unwrap(),
                                        });
                                        continue;
                                    }
                                    (Ok(val), Lit::Int(intlit))
                                        if val == "NOT_EQUAL_TO".to_string()
                                            || val == "DIFFERENT_FROM".to_string()
                                            || val == "NEQ".to_string() =>
                                    {
                                        reqs.push(BitFieldRequest::SizeNeq {
                                            cmp_with: intlit.base10_parse().unwrap(),
                                        });
                                        continue;
                                    }
                                    (Ok(val), Lit::Int(intlit))
                                        if val == "LESS_THAN".to_string()
                                            || val == "LT".to_string() =>
                                    {
                                        reqs.push(BitFieldRequest::SizeLt {
                                            cmp_with: intlit.base10_parse().unwrap(),
                                        });
                                        continue;
                                    }
                                    (Ok(val), Lit::Int(intlit)) if val == "LTE".to_string() =>
                                    {
                                        reqs.push(BitFieldRequest::SizeLte {
                                            cmp_with: intlit.base10_parse().unwrap(),
                                        });
                                        continue;
                                    }
                                    (Ok(val), Lit::Int(intlit))
                                        if val == "GREATER_THAN".to_string()
                                            || val == "GT".to_string() =>
                                    {
                                        reqs.push(BitFieldRequest::SizeGt {
                                            cmp_with: intlit.base10_parse().unwrap(),
                                        });
                                        continue;
                                    }
                                    (Ok(val), Lit::Int(intlit)) if val == "GTE".to_string() =>
                                    {
                                        reqs.push(BitFieldRequest::SizeGte {
                                            cmp_with: intlit.base10_parse().unwrap(),
                                        });
                                        continue;
                                    }
                                    _ =>
                                    {}
                                }
                            }
                            _ =>
                            {}
                        }
                    }
                }
            }
            others.push(attr.clone());
        }
        (reqs, others)
    }
}

impl ToTokens for BitFieldDecl
{
    fn to_tokens(&self, tokens: &mut TokenStream)
    {
        let (our_attrs, other_attrs) = self.handle_attrs();
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
            #(#other_attrs)*
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
        for attr in our_attrs
        {
            tokens.extend(attr.expand(name, &range_base, &underlying_type));
        }
        let underlying_sz = &self.underlying_size;
        tokens.extend(
            make_static_assert_cmp(range_base, underlying_sz, quote! { <= }).unwrap_or_else(
                |err| {
                    let errstr = format!("{err}\n");
                    quote! { compile_error!(#errstr) }
                },
            ),
        );
    }
}

fn impl_new_for(struct_name: &Ident) -> TokenStream
{
    quote! {
        impl #struct_name {
            pub const fn new() -> Self {
                unsafe {
                    core::mem::zeroed()
                }
            }
        }
    }
}

fn impl_default_for(struct_name: &Ident) -> TokenStream
{
    quote! {
        impl Default for #struct_name {
            fn default() -> Self {
                unsafe {
                    core::mem::zeroed()
                }
            }
        }
    }
}

fn impl_as_ref_for(struct_name: &Ident, underlying_type: &Type) -> TokenStream
{
    quote! {
        impl AsRef<#underlying_type> for #struct_name {
            fn as_ref(&self) -> &#underlying_type {
                &self.0
            }
        }
    }
}

fn impl_as_mut_for(struct_name: &Ident, underlying_type: &Type) -> TokenStream
{
    quote! {
        impl AsMut<#underlying_type> for #struct_name {
            fn as_mut(&mut self) -> &mut #underlying_type {
                &mut self.0
            }
        }
    }
}

#[proc_macro]
pub fn bitfield(input: TokenStreamClassic) -> TokenStreamClassic
{
    let parsed = parse_macro_input!(input as BitFieldDecl);

    parsed.into_token_stream().into()
}

struct StaticAssertArgs
{
    condition: Expr,
    msg: LitStr,
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
                msg: input.parse()?,
            })
        }
        else
        {
            Ok(Self {
                msg: {
                    let failed_string = format!(
                        "static assertion `{}` failed",
                        cond.clone().to_token_stream().to_string()
                    );
                    syn::parse2(quote! { #failed_string }).expect("internal error !")
                },
                condition: cond,
            })
        }
    }
}

fn make_static_assert<T, U>(expr: T, msg: Option<U>) -> Result<TokenStream, getrandom::Error>
where
    T: ToTokens,
    U: ToTokens,
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
    fn to_tokens(&self, tokens: &mut TokenStream)
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
pub fn static_assert(input: TokenStreamClassic) -> TokenStreamClassic
{
    let parsed = parse_macro_input!(input as StaticAssertArgs);

    quote! { #parsed }.into()
}

//#[cfg(test)]
//mod tests
//{
//    use syn::parse_str;
//
//    use crate::BitFieldElemUnion;
//
//    #[test]
//    fn union_test()
//    {
//        let to_parse = r#"
//            union {
//                pub u8 test1: 4;
//                pub u8 test2: 4;
//            };"#;
//        let union_: BitFieldElemUnion = parse_str(to_parse).unwrap_or_else(|err| {
//            println!("{}", err.clone());
//            err
//        })?;
//    }
//}
