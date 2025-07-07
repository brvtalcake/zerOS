#![allow(non_snake_case)]

use std::{mem::MaybeUninit, str::FromStr};

use num_traits::FromPrimitive;
use proc_macro::TokenStream;
use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote};
use syn::{
	Attribute,
	Expr,
	ExprAssign,
	Ident,
	Lit,
	Token,
	Type,
	braced,
	parse::{Parse, ParseStream},
	parse_macro_input,
	parse_str,
	punctuated::Punctuated,
	spanned::Spanned
};

trait AsTokenStream
{
	fn as_token_stream(
		&self,
		struct_name: &Ident,
		range_base: &Expr,
		size_asserts: &mut TokenStream2
	) -> TokenStream2;
}

// trait CompTimeEvaluator
//{
// 	fn comptime_evaluable_as<T: FromStr>(expr: &Expr) -> bool
// 	where
// 		T::Err: std::fmt::Display;
//
// 	fn comptime_evaluate_as<T: FromStr>(expr: &Expr) -> Option<T>
// 	where
// 		T::Err: std::fmt::Display;
//}

fn comptime_evaluable_as<T: FromStr + FromPrimitive>(expr: &Expr) -> Option<T>
where
	T::Err: std::fmt::Display
{
	match expr
	{
		Expr::Lit(litexpr) =>
		{
			match &litexpr.lit
			{
				Lit::Int(lit) => lit.base10_parse().ok(),
				Lit::Float(lit) => lit.base10_parse::<f64>().ok().and_then(T::from_f64),
				_ => None
			}
		},
		_ => None
	}
}

#[allow(dead_code)]
#[derive(Debug)]
struct BitFieldElemOnly
{
	vis:   Option<Token![pub]>,
	ty:    Type,
	ident: Ident,
	sep:   Token![:],
	size:  Expr
}

impl Parse for BitFieldElemOnly
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		Ok(Self {
			vis:   input.parse().unwrap_or(None),
			ty:    input.parse()?,
			ident: input.parse()?,
			sep:   input.parse()?,
			size:  input.parse()?
		})
	}
}

fn select_suitable_type(size: &Expr) -> Option<Type>
{
	let parsed_size = comptime_evaluable_as::<usize>(size)?;
	if parsed_size <= 8
	{
		syn::parse2(quote! { u8 }).ok()
	}
	else if parsed_size <= 16
	{
		syn::parse2(quote! { u16 }).ok()
	}
	else if parsed_size <= 32
	{
		syn::parse2(quote! { u32 }).ok()
	}
	else if parsed_size <= 64
	{
		syn::parse2(quote! { u64 }).ok()
	}
	else if parsed_size <= 128
	{
		syn::parse2(quote! { u128 }).ok()
	}
	else
	{
		None
	}
}

impl AsTokenStream for BitFieldElemOnly
{
	fn as_token_stream(
		&self,
		struct_name: &Ident,
		range_base: &Expr,
		size_asserts: &mut TokenStream2
	) -> TokenStream2
	{
		let ident = &self.ident;
		let sz = &self.size;
		let retty = match &self.ty
		{
			Type::Infer(_) =>
			{
				let maybe_suitable = select_suitable_type(sz);
				if maybe_suitable.is_none()
				{
					let errstr = format!(
						"size value {} of field {} in bitfield {} must be known at compile time \
						 and less than 128 to be able to automatically select a suitable type",
						sz.to_token_stream().to_string(),
						ident.to_token_stream().to_string(),
						struct_name.to_token_stream().to_string()
					);
					return syn::Error::new(self.size.span(), errstr).into_compile_error();
				}
				maybe_suitable.unwrap()
			},
			provided =>
			{
				size_asserts.extend(make_static_assert_cmp(
					quote! {
						::core::mem::size_of::<#provided>() * 8
					},
					quote! {
						#sz
					},
					quote! { >= }
				));
				provided.clone()
			}
		};
		let vi = if let Some(visibility) = self.vis
		{
			visibility.into_token_stream()
		}
		else
		{
			TokenStream2::new()
		};
		let getfn = format_ident!("get_{}", ident);
		let setfn = format_ident!("set_{}", ident);
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
	ElemStruct(BitFieldElemInnerStruct)
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
	fn as_token_stream(
		&self,
		struct_name: &Ident,
		range_base: &Expr,
		size_asserts: &mut TokenStream2
	) -> TokenStream2
	{
		match self
		{
			Self::ElemOnly(elem) => elem.as_token_stream(struct_name, range_base, size_asserts),
			Self::ElemStruct(elem) => elem.as_token_stream(struct_name, range_base, size_asserts)
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
	kw:     Token![struct],
	braces: syn::token::Brace,
	elems:  Punctuated<BitFieldElemOnly, Token![;]>
}

impl AsTokenStream for BitFieldElemInnerStruct
{
	fn as_token_stream(
		&self,
		struct_name: &Ident,
		range_base: &Expr,
		size_asserts: &mut TokenStream2
	) -> TokenStream2
	{
		let mut stream: TokenStream2 = TokenStream2::new();
		let mut new_range_base = range_base.clone();
		for elem in &self.elems
		{
			stream.extend(elem.as_token_stream(struct_name, &new_range_base, size_asserts));

			let elem_size = &elem.size;
			let new_size = quote! { (#new_range_base) + (#elem_size) };

			new_range_base = match syn::parse2(new_size)
			{
				Ok(sz) => sz,
				Err(_) => unreachable!()
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
			kw:     input.parse()?,
			braces: braced!(content in input),
			elems:  <Punctuated<_, _>>::parse_terminated(&content)?
		})
	}
}

#[allow(dead_code)]
#[derive(Debug)]
struct BitFieldElemUnion
{
	kw:     Token![union],
	braces: syn::token::Brace,
	elems:  Punctuated<BitFieldElemInner, Token![;]>
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
	fn as_token_stream(
		&self,
		struct_name: &Ident,
		range_base: &Expr,
		size_asserts: &mut TokenStream2
	) -> TokenStream2
	{
		let mut stream: TokenStream2 = TokenStream2::new();
		for elem in &self.elems
		{
			stream.extend(elem.as_token_stream(struct_name, range_base, size_asserts));
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
			kw:     input.parse()?,
			braces: braced!(content in input),
			elems:  <Punctuated<_, _>>::parse_terminated(&content)?
		})
	}
}

#[derive(Debug)]
enum BitFieldElem
{
	ElemOnly(BitFieldElemOnly),
	ElemUnion(BitFieldElemUnion)
}

impl BitFieldElem
{
	fn size(&self) -> Expr
	{
		match self
		{
			Self::ElemOnly(elem) => elem.size.clone(),
			Self::ElemUnion(elem) => elem.size()
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
	fn as_token_stream(
		&self,
		struct_name: &Ident,
		range_base: &Expr,
		size_asserts: &mut TokenStream2
	) -> TokenStream2
	{
		match self
		{
			Self::ElemOnly(elem) => elem.as_token_stream(struct_name, range_base, size_asserts),
			Self::ElemUnion(elem) => elem.as_token_stream(struct_name, range_base, size_asserts)
		}
	}
}

#[allow(dead_code)]
#[derive(Debug)]
struct BitFieldBlockOuter
{
	braces: syn::token::Brace,
	elems:  Punctuated<BitFieldElem, Token![;]>
}
impl Parse for BitFieldBlockOuter
{
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
	{
		let content;
		Ok(Self {
			braces: braced!(content in input),
			elems:  <Punctuated<BitFieldElem, Token![;]>>::parse_terminated(&content)?
		})
	}
}

#[allow(dead_code)]
#[derive(Debug)]
struct BitFieldDecl
{
	attrs:           Vec<syn::Attribute>,
	/// TODO: this should be `syn::Visibility`
	vis:             Option<Token![pub]>,
	kw:              Token![struct],
	name:            syn::Ident,
	arrow:           Token![->],
	underlying_size: syn::LitInt,
	block:           BitFieldBlockOuter
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
			attrs:           input.call(syn::Attribute::parse_outer)?,
			vis:             input.parse().unwrap_or(None),
			kw:              input.parse()?,
			name:            input.parse()?,
			arrow:           input.parse()?,
			underlying_size: input.parse()?,
			block:           input.parse()?
		})
	}
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

fn make_static_assert_cmp<T, U>(
	lhs: T,
	rhs: U,
	cmpsym: TokenStream2
) -> Result<TokenStream2, getrandom::Error>
where
	T: ToTokens,
	U: ToTokens
{
	let modname = make_random_ident(Some("__random_module_for_static_assert_"), Some("__"))?;
	let fnname = make_random_ident(Some("__random_fn_for_static_assert_"), Some("__"))?;
	Ok(quote! {
		mod #modname
		{
			#[allow(dead_code)]
			const fn #fnname() {
				assert!(((#lhs) #cmpsym (#rhs)));
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
	}
}

impl BitFieldRequest
{
	fn expand(
		&self,
		struct_name: &Ident,
		summed_size: &Expr,
		underlying_type: &Type
	) -> TokenStream2
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
					TokenStream2::new()
				}
			},
			Self::Default { impl_it } =>
			{
				if *impl_it
				{
					impl_default_for(struct_name)
				}
				else
				{
					TokenStream2::new()
				}
			},
			Self::AsRef { impl_it } =>
			{
				if *impl_it
				{
					impl_as_ref_for(struct_name, underlying_type)
				}
				else
				{
					TokenStream2::new()
				}
			},
			Self::AsMut { impl_it } =>
			{
				if *impl_it
				{
					impl_as_mut_for(struct_name, underlying_type)
				}
				else
				{
					TokenStream2::new()
				}
			},
			Self::SizeEq { cmp_with } =>
			{
				make_static_assert_cmp(summed_size, cmp_with, quote! { == }).unwrap()
			},
			Self::SizeNeq { cmp_with } =>
			{
				make_static_assert_cmp(summed_size, cmp_with, quote! { != }).unwrap()
			},
			Self::SizeGt { cmp_with } =>
			{
				make_static_assert_cmp(summed_size, cmp_with, quote! { > }).unwrap()
			},
			Self::SizeGte { cmp_with } =>
			{
				make_static_assert_cmp(summed_size, cmp_with, quote! { >= }).unwrap()
			},
			Self::SizeLt { cmp_with } =>
			{
				make_static_assert_cmp(summed_size, cmp_with, quote! { < }).unwrap()
			},
			Self::SizeLte { cmp_with } =>
			{
				make_static_assert_cmp(summed_size, cmp_with, quote! { <= }).unwrap()
			},
		}
	}
}

fn parse_yesno(lit: &Lit) -> Result<bool, syn::Error>
{
	match lit
	{
		Lit::Bool(val) => Ok(val.value()),
		Lit::Byte(val) => Ok(val.value() != b'0'),
		Lit::ByteStr(val) =>
		{
			Ok({
				let ascii = val.value().to_ascii_uppercase();
				ascii == b"YES" || ascii == b"ALWAYS" || ascii == b"TRUE" || ascii == b"ON"
			})
		},
		Lit::CStr(val) =>
		{
			Ok({
				let ascii = val.value().to_bytes().to_ascii_uppercase();
				ascii == b"YES" || ascii == b"ALWAYS" || ascii == b"TRUE" || ascii == b"ON"
			})
		},
		Lit::Char(val) => Ok(val.value() != '0'),
		Lit::Float(val) =>
		{
			Ok({
				let float: f64 = val.base10_parse()?;
				float.round() != 0.0
			})
		},
		Lit::Int(val) =>
		{
			Ok({
				let integer: i128 = val.base10_parse()?;
				integer != 0
			})
		},
		Lit::Str(val) =>
		{
			Ok(val.value() == "YES"
				|| val.value() == "ALWAYS"
				|| val.value() == "TRUE"
				|| val.value() == "ON")
		},
		_ => Err(syn::Error::new(Span2::call_site(), "invalid literal"))
	}
}

impl BitFieldDecl
{
	fn handle_provide(lhs: Box<Expr>, rhs: Box<Expr>, reqs: &mut Vec<BitFieldRequest>) -> bool
	{
		if let Expr::Path(lpath) = lhs.as_ref()
		{
			if let Expr::Lit(rlit) = rhs.as_ref()
			{
				if let Ok(as_str) = lpath.path.require_ident().map(|ok| ok.to_string())
				{
					match as_str.to_uppercase().as_str()
					{
						"CONSTRUCTOR" | "NEW" | "CTOR" =>
						{
							if let Ok(yesno) = parse_yesno(&rlit.lit)
							{
								if let Some(elem) = reqs.iter_mut().find(|el| {
									match el
									{
										BitFieldRequest::Constructor { impl_it: _ } => true,
										_ => false
									}
								})
								{
									*elem = BitFieldRequest::Constructor { impl_it: yesno };
									return true;
								}
								else
								{
									reqs.push(BitFieldRequest::Constructor { impl_it: yesno });
									return true;
								}
							}
						},
						"DEFAULT" =>
						{
							if let Ok(yesno) = parse_yesno(&rlit.lit)
							{
								if let Some(elem) = reqs.iter_mut().find(|el| {
									match el
									{
										BitFieldRequest::Default { impl_it: _ } => true,
										_ => false
									}
								})
								{
									*elem = BitFieldRequest::Default { impl_it: yesno };
									return true;
								}
								else
								{
									reqs.push(BitFieldRequest::Default { impl_it: yesno });
									return true;
								}
							}
						},
						"ASREF" | "AS_REF" =>
						{
							if let Ok(yesno) = parse_yesno(&rlit.lit)
							{
								if let Some(elem) = reqs.iter_mut().find(|el| {
									match el
									{
										BitFieldRequest::AsRef { impl_it: _ } => true,
										_ => false
									}
								})
								{
									*elem = BitFieldRequest::AsRef { impl_it: yesno };
									return true;
								}
								else
								{
									reqs.push(BitFieldRequest::AsRef { impl_it: yesno });
									return true;
								}
							}
						},
						"ASMUT" | "AS_MUT" =>
						{
							if let Ok(yesno) = parse_yesno(&rlit.lit)
							{
								if let Some(elem) = reqs.iter_mut().find(|el| {
									match el
									{
										BitFieldRequest::AsMut { impl_it: _ } => true,
										_ => false
									}
								})
								{
									*elem = BitFieldRequest::AsMut { impl_it: yesno };
									return true;
								}
								else
								{
									reqs.push(BitFieldRequest::AsMut { impl_it: yesno });
									return true;
								}
							}
						},
						_ =>
						{}
					}
				}
			}
		}
		false
	}

	fn handle_check(lhs: Box<Expr>, rhs: Box<Expr>, reqs: &mut Vec<BitFieldRequest>) -> bool
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
					&lit.lit
				)
				{
					(Ok(val), Lit::Int(intlit))
						if val == "EQUAL_TO".to_string() || val == "EQ".to_string() =>
					{
						reqs.push(BitFieldRequest::SizeEq {
							cmp_with: intlit.base10_parse().unwrap()
						});
						return true;
					},
					(Ok(val), Lit::Int(intlit))
						if val == "NOT_EQUAL_TO".to_string()
							|| val == "DIFFERENT_FROM".to_string()
							|| val == "NEQ".to_string() =>
					{
						reqs.push(BitFieldRequest::SizeNeq {
							cmp_with: intlit.base10_parse().unwrap()
						});
						return true;
					},
					(Ok(val), Lit::Int(intlit))
						if val == "LESS_THAN".to_string() || val == "LT".to_string() =>
					{
						reqs.push(BitFieldRequest::SizeLt {
							cmp_with: intlit.base10_parse().unwrap()
						});
						return true;
					},
					(Ok(val), Lit::Int(intlit)) if val == "LTE".to_string() =>
					{
						reqs.push(BitFieldRequest::SizeLte {
							cmp_with: intlit.base10_parse().unwrap()
						});
						return true;
					},
					(Ok(val), Lit::Int(intlit))
						if val == "GREATER_THAN".to_string() || val == "GT".to_string() =>
					{
						reqs.push(BitFieldRequest::SizeGt {
							cmp_with: intlit.base10_parse().unwrap()
						});
						return true;
					},
					(Ok(val), Lit::Int(intlit)) if val == "GTE".to_string() =>
					{
						reqs.push(BitFieldRequest::SizeGte {
							cmp_with: intlit.base10_parse().unwrap()
						});
						return true;
					},
					_ =>
					{}
				}
			},
			_ =>
			{}
		}
		false
	}

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
						left: lhs,
						right: rhs,
						..
					}) = attr.parse_args().ok()
					{
						if Self::handle_provide(lhs, rhs, &mut reqs)
						{
							continue;
						}
					}
				}
				else if ident.to_string().to_uppercase() == "CHECK"
				{
					if let Some(ExprAssign {
						left: lhs,
						right: rhs,
						..
					}) = attr.parse_args().ok()
					{
						if Self::handle_check(lhs, rhs, &mut reqs)
						{
							continue;
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
	fn to_tokens(&self, tokens: &mut TokenStream2)
	{
		let mut size_asserts = TokenStream2::new();
		let (our_attrs, other_attrs) = self.handle_attrs();
		let vis = if let Some(visibility) = self.vis
		{
			visibility.into_token_stream()
		}
		else
		{
			TokenStream2::new()
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
			let tokstream = decl.as_token_stream(name, &range_base, &mut size_asserts);
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
				}
			)
		);
		tokens.extend(size_asserts);
	}
}

fn impl_new_for(struct_name: &Ident) -> TokenStream2
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

fn impl_default_for(struct_name: &Ident) -> TokenStream2
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

fn impl_as_ref_for(struct_name: &Ident, underlying_type: &Type) -> TokenStream2
{
	quote! {
		impl AsRef<#underlying_type> for #struct_name {
			fn as_ref(&self) -> &#underlying_type {
				&self.0
			}
		}
	}
}

fn impl_as_mut_for(struct_name: &Ident, underlying_type: &Type) -> TokenStream2
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
pub fn bitfield(input: TokenStream) -> TokenStream
{
	let parsed = parse_macro_input!(input as BitFieldDecl);

	parsed.into_token_stream().into()
}
