//! # proc-macro-utils
//!
//! This crate defines some utility proc_macro's that are
//! mainly used in the zerOS kernel.
//! For example, it defines a `bitfield!` macro, that can be
//! used like so:
//! ```rust
//! bitfield! {
//! 	pub struct GDTDescriptor -> 64
//! 	{
//! 		pub u16 base_low: 12;
//! 		u16 blah: 9;
//! 		...
//! 		union
//! 		{
//! 			pub u8 access: 8;
//! 			struct
//! 			{
//! 				// detailed fields
//! 			};
//! 		};
//! 	}
//! }
//! ```

#![feature(proc_macro_expand)]
#![feature(proc_macro_totokens)]
#![feature(min_specialization)]
#![feature(f128)]
#![feature(iter_collect_into)]
#![feature(write_all_vectored)]

extern crate proc_macro;

use std::{
	env,
	ffi::OsStr,
	fs,
	io::{IoSlice, Write},
	mem::{self, MaybeUninit},
	os::unix::fs::PermissionsExt,
	path::{self},
	process::{self, Command},
	str::FromStr
};

use heck::{
	ToKebabCase,
	ToLowerCamelCase,
	ToShoutyKebabCase,
	ToShoutySnakeCase,
	ToSnakeCase,
	ToTitleCase,
	ToTrainCase,
	ToUpperCamelCase
};
use num_traits::FromPrimitive;
use proc_macro::TokenStream as TokenStreamClassic;
use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::{
	Attribute,
	DeriveInput,
	Expr,
	ExprAssign,
	Fields,
	Generics,
	Ident,
	ItemEnum,
	Lit,
	LitStr,
	StaticMutability,
	Stmt,
	Token,
	Type,
	TypeArray,
	Visibility,
	braced,
	ext::IdentExt,
	parse::{Parse, ParseStream, discouraged::Speculative},
	parse_macro_input,
	parse_str,
	punctuated::Punctuated,
	spanned::Spanned,
	token::{Brace, Paren}
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
	custom_keyword!(body);
	// custom_keyword!(bitfield);
	// custom_keyword!(bits);
}

mod punct
{
	use syn::custom_punctuation;
	custom_punctuation!(AtSign, @);
}

trait AsTokenStream
{
	fn as_token_stream(
		&self,
		struct_name: &Ident,
		range_base: &Expr,
		size_asserts: &mut TokenStream
	) -> TokenStream;
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
		size_asserts: &mut TokenStream
	) -> TokenStream
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
			TokenStream::new()
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
		size_asserts: &mut TokenStream
	) -> TokenStream
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
		size_asserts: &mut TokenStream
	) -> TokenStream
	{
		let mut stream: TokenStream = TokenStream::new();
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
		size_asserts: &mut TokenStream
	) -> TokenStream
	{
		let mut stream: TokenStream = TokenStream::new();
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
		size_asserts: &mut TokenStream
	) -> TokenStream
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
	cmpsym: TokenStream
) -> Result<TokenStream, getrandom::Error>
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
			},
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
			},
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
			},
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
		_ => Err(syn::Error::new(Span::call_site(), "invalid literal"))
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
						right: rhs
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
												if let Some(elem) = reqs.iter_mut().find(|el| {
													match el
													{
														BitFieldRequest::Constructor {
															impl_it: _
														} => true,
														_ => false
													}
												})
												{
													*elem = BitFieldRequest::Constructor {
														impl_it: yesno
													};
													continue;
												}
												else
												{
													reqs.push(BitFieldRequest::Constructor {
														impl_it: yesno
													});
													continue;
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
														BitFieldRequest::Default { impl_it: _ } =>
														{
															true
														},
														_ => false
													}
												})
												{
													*elem =
														BitFieldRequest::Default { impl_it: yesno };
													continue;
												}
												else
												{
													reqs.push(BitFieldRequest::Default {
														impl_it: yesno
													});
													continue;
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
														BitFieldRequest::AsRef { impl_it: _ } =>
														{
															true
														},
														_ => false
													}
												})
												{
													*elem =
														BitFieldRequest::AsRef { impl_it: yesno };
													continue;
												}
												else
												{
													reqs.push(BitFieldRequest::AsRef {
														impl_it: yesno
													});
													continue;
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
														BitFieldRequest::AsMut { impl_it: _ } =>
														{
															true
														},
														_ => false
													}
												})
												{
													*elem =
														BitFieldRequest::AsMut { impl_it: yesno };
													continue;
												}
												else
												{
													reqs.push(BitFieldRequest::AsMut {
														impl_it: yesno
													});
													continue;
												}
											}
										},
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
						right: rhs
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
									&lit.lit
								)
								{
									(Ok(val), Lit::Int(intlit))
										if val == "EQUAL_TO".to_string()
											|| val == "EQ".to_string() =>
									{
										reqs.push(BitFieldRequest::SizeEq {
											cmp_with: intlit.base10_parse().unwrap()
										});
										continue;
									},
									(Ok(val), Lit::Int(intlit))
										if val == "NOT_EQUAL_TO".to_string()
											|| val == "DIFFERENT_FROM".to_string()
											|| val == "NEQ".to_string() =>
									{
										reqs.push(BitFieldRequest::SizeNeq {
											cmp_with: intlit.base10_parse().unwrap()
										});
										continue;
									},
									(Ok(val), Lit::Int(intlit))
										if val == "LESS_THAN".to_string()
											|| val == "LT".to_string() =>
									{
										reqs.push(BitFieldRequest::SizeLt {
											cmp_with: intlit.base10_parse().unwrap()
										});
										continue;
									},
									(Ok(val), Lit::Int(intlit)) if val == "LTE".to_string() =>
									{
										reqs.push(BitFieldRequest::SizeLte {
											cmp_with: intlit.base10_parse().unwrap()
										});
										continue;
									},
									(Ok(val), Lit::Int(intlit))
										if val == "GREATER_THAN".to_string()
											|| val == "GT".to_string() =>
									{
										reqs.push(BitFieldRequest::SizeGt {
											cmp_with: intlit.base10_parse().unwrap()
										});
										continue;
									},
									(Ok(val), Lit::Int(intlit)) if val == "GTE".to_string() =>
									{
										reqs.push(BitFieldRequest::SizeGte {
											cmp_with: intlit.base10_parse().unwrap()
										});
										continue;
									},
									_ =>
									{}
								}
							},
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
		let mut size_asserts = TokenStream::new();
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
	msg:       LitStr
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

fn make_static_assert<T, U>(expr: T, msg: Option<U>) -> Result<TokenStream, getrandom::Error>
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

#[allow(dead_code)]
struct ItemConstRunnable
{
	attrs:       Vec<Attribute>,
	vis:         Visibility,
	const_token: Token![const],
	ident:       Ident,
	generics:    Generics,
	colon_token: Token![:],
	ty:          Box<Type>,
	eq_token:    Token![=],
	body:        kw::body,
	exclam:      Token![!],
	braces:      Brace,
	code:        TokenStream,
	semi_token:  Token![;]
}

macro_rules! get_braced_raw {
	($buf:ident in $input:ident) => {{
		$buf = braced_raw($input)?;
		Brace(Span::call_site())
	}};
}

macro_rules! get_parenthesized_raw {
	($buf:ident in $input:ident) => {{
		$buf = parenthesized_raw($input)?;
		Paren(Span::call_site())
	}};
}

macro_rules! expect_delimited_group_raw {
	($input:ident, $delim:ident) => {
		$input.step(|cursor| {
			let rest = *cursor;
			if let Some((tt, next)) = rest.token_tree()
			{
				match &tt
				{
					TokenTree::Group(delim) if matches!(delim.delimiter(), Delimiter::$delim) =>
					{
						return Ok((delim.stream(), next));
					},
					_ =>
					{}
				}
			}
			Err(cursor.error(format!(
				"expected {}-delimited group",
				String::from(stringify!($delim)).to_lowercase()
			)))
		})
	};
}

fn braced_raw(input: ParseStream) -> syn::Result<TokenStream>
{
	expect_delimited_group_raw!(input, Brace)
}

fn parenthesized_raw(input: ParseStream) -> syn::Result<TokenStream>
{
	expect_delimited_group_raw!(input, Parenthesis)
}

impl Parse for ItemConstRunnable
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		let code_content;
		Ok(Self {
			attrs:       input.call(Attribute::parse_outer)?,
			vis:         input.parse()?,
			const_token: input.parse()?,
			ident:       input.parse()?,
			generics:    input.parse()?,
			colon_token: input.parse()?,
			ty:          input.parse()?,
			eq_token:    input.parse()?,
			body:        input.parse()?,
			exclam:      input.parse()?,
			braces:      get_braced_raw!(code_content in input),
			code:        code_content,
			semi_token:  input.parse()?
		})
	}
}

#[allow(dead_code)]
struct ItemStaticRunnable
{
	attrs:        Vec<Attribute>,
	vis:          Visibility,
	static_token: Token![static],
	mutability:   StaticMutability,
	ident:        Ident,
	colon_token:  Token![:],
	ty:           Box<Type>,
	eq_token:     Token![=],
	body:         kw::body,
	exclam:       Token![!],
	braces:       Brace,
	code:         TokenStream,
	semi_token:   Token![;]
}

impl Parse for ItemStaticRunnable
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		let code_content;
		Ok(Self {
			attrs:        input.call(Attribute::parse_outer)?,
			vis:          input.parse()?,
			static_token: input.parse()?,
			mutability:   input.parse()?,
			ident:        input.parse()?,
			colon_token:  input.parse()?,
			ty:           input.parse()?,
			eq_token:     input.parse()?,
			body:         input.parse()?,
			exclam:       input.parse()?,
			braces:       get_braced_raw!(code_content in input),
			code:         code_content,
			semi_token:   input.parse()?
		})
	}
}

enum RunnableItem
{
	Const(ItemConstRunnable),
	Static(ItemStaticRunnable)
}

enum RunnableLang
{
	C
	{
		compiler: String,
		std:      String,
		cc_opts:  Vec<String>
	},
	Cxx
	{
		compiler: String,
		std:      String,
		cc_opts:  Vec<String>
	},
	Rust
	{
		compiler:   String,
		rustc_opts: Vec<String>
	},
	Shell
	{
		shell:      String,
		shell_opts: Vec<String>
	}
}

impl RunnableLang
{
	fn format_extension(&self) -> &'_ str
	{
		match self
		{
			Self::C { .. } => "c",
			Self::Cxx { .. } => "cc",
			Self::Rust { .. } => "rs",
			Self::Shell { shell, .. } => shell.as_str()
		}
	}

	fn set_std<'a, T>(&mut self, new_std: Option<&'a T>) -> &mut Self
	where
		String: From<&'a T>
	{
		match new_std
		{
			Some(new_value) =>
			{
				if let Self::C { ref mut std, .. } = *self
				{
					*std = new_value.into();
				}
				else if let Self::Cxx { ref mut std, .. } = *self
				{
					*std = new_value.into();
				}
				self
			},
			_ => self
		}
	}

	fn set_cc_opts(&mut self, new_cc_opts: &Vec<String>) -> &mut Self
	{
		if let Self::C {
			ref mut cc_opts, ..
		} = *self
		{
			*cc_opts = new_cc_opts.iter().map(|el| el.into()).collect();
		}
		else if let Self::Cxx {
			ref mut cc_opts, ..
		} = *self
		{
			*cc_opts = new_cc_opts.iter().map(|el| el.into()).collect();
		}
		self
	}

	fn set_rustc_opts(&mut self, new_rustc_opts: &Vec<String>) -> &mut Self
	{
		if let Self::Rust {
			ref mut rustc_opts, ..
		} = *self
		{
			*rustc_opts = new_rustc_opts.iter().map(|el| el.into()).collect();
		}
		self
	}

	fn set_shell_opts(&mut self, new_shell_opts: &Vec<String>) -> &mut Self
	{
		if let Self::Shell {
			ref mut shell_opts, ..
		} = *self
		{
			*shell_opts = new_shell_opts.iter().map(|el| el.into()).collect();
		}
		self
	}

	fn set_shell<'a, T>(&mut self, new_shell: Option<&'a T>) -> &mut Self
	where
		String: From<&'a T>
	{
		match new_shell
		{
			Some(new_value) =>
			{
				if let Self::Shell { ref mut shell, .. } = *self
				{
					*shell = new_value.into();
				}
				self
			},
			_ => self
		}
	}

	fn set_compiler<'a, T>(&mut self, new_compiler: Option<&'a T>) -> &mut Self
	where
		String: From<&'a T>
	{
		match new_compiler
		{
			Some(new_value) =>
			{
				if let Self::C {
					ref mut compiler, ..
				} = *self
				{
					*compiler = new_value.into();
				}
				else if let Self::Cxx {
					ref mut compiler, ..
				} = *self
				{
					*compiler = new_value.into();
				}
				else if let Self::Rust {
					ref mut compiler, ..
				} = *self
				{
					*compiler = new_value.into();
				}
				self
			},
			_ => self
		}
	}

	fn default_for(lang: &'_ str) -> Result<Self, String>
	{
		match lang
		{
			"C" =>
			{
				Ok(Self::C {
					compiler: "gcc".into(),
					std:      "gnu23".into(),
					cc_opts:  vec![]
				})
			},
			"C++" | "CXX" =>
			{
				Ok(Self::Cxx {
					compiler: "g++".into(),
					std:      "gnu++23".into(),
					cc_opts:  vec![]
				})
			},
			"RUST" =>
			{
				Ok(Self::Rust {
					compiler:   "rustc".into(),
					rustc_opts: vec![]
				})
			},
			"SHELL" =>
			{
				Ok(Self::Shell {
					shell:      "bash".into(),
					shell_opts: vec![]
				})
			},
			_ => Err(format!("unknown language {lang}"))
		}
	}
}

enum RunnableExpandKind
{
	Output,
	StdOut,
	StdErr,
	ExitCode
}

struct RunnableArgs
{
	lang:   RunnableLang,
	args:   Vec<String>,
	expand: RunnableExpandKind
}

impl Parse for RunnableArgs
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		// #[runnable(lang(c++), std(gnu++23), args(blah blah) )]
		let mut comp_or_interp = None;
		let mut comp_or_interp_options = vec![];
		let mut language = RunnableLang::default_for("SHELL").unwrap();
		let mut standard = None;
		let mut exe_args = vec![];
		let mut found_lang = false;
		let mut expand = None;

		loop
		{
			let ident = input.call(Ident::parse_any).unwrap();
			match ident.to_string().to_uppercase().as_str()
			{
				"EXPAND" =>
				{
					if expand.is_some()
					{
						return Err(syn::Error::new_spanned(
							ident,
							"the expand parameter must either be supplied only once or never"
						));
					}
					let buf;
					get_parenthesized_raw!(buf in input);
					expand = Some(match buf.to_string().to_uppercase().as_str()
					{
						"OUTPUT" => RunnableExpandKind::Output,
						"STDOUT" => RunnableExpandKind::StdOut,
						"STDERR" => RunnableExpandKind::StdErr,
						"EXITCODE" | "EXIT_CODE" => RunnableExpandKind::ExitCode,
						val =>
						{
							return Err(syn::Error::new_spanned(
								ident,
								format!("unknown expand parameter value `{}`", val)
							));
						}
					});
				},
				"LANG" | "LANGUAGE" =>
				{
					if found_lang
					{
						return Err(syn::Error::new_spanned(
							ident,
							"language parameter must be specified only once"
						));
					}
					let buf;
					get_parenthesized_raw!(buf in input);
					let default =
						RunnableLang::default_for(buf.to_string().to_uppercase().as_str())
							.map_err(|s| syn::Error::new_spanned(ident, s))?;
					found_lang = true;
					language = default;
					language
						.set_cc_opts(&comp_or_interp_options)
						.set_rustc_opts(&comp_or_interp_options)
						.set_shell_opts(&comp_or_interp_options)
						.set_std(standard.as_ref())
						.set_shell(comp_or_interp.as_ref())
						.set_compiler(comp_or_interp.as_ref());
				},
				"STD" | "STANDARD" =>
				{
					if standard.is_some()
					{
						return Err(syn::Error::new_spanned(
							ident,
							"the standard parameter must either be supplied only once or never"
						));
					}
					let buf;
					get_parenthesized_raw!(buf in input);
					standard = Some(buf.to_string());
					if found_lang
					{
						language.set_std(standard.as_ref());
					}
				},
				"CCOPTS"
				| "CC_OPTS"
				| "RUSTC_OPTS"
				| "RUSTCOPTS"
				| "SH_OPTS"
				| "SHOPTS"
				| "SHELL_OPTS"
				| "SHELLOPTS"
				| "COMPILER_OPTIONS"
				| "INTERPRETER_OPTIONS" =>
				{
					let buf;
					get_parenthesized_raw!(buf in input);
					comp_or_interp_options.extend(
						buf.to_string()
							.split_whitespace()
							.map(|s| s.to_string())
							.collect::<Vec<_>>()
					);
					if found_lang
					{
						language
							.set_cc_opts(&comp_or_interp_options)
							.set_rustc_opts(&comp_or_interp_options)
							.set_shell_opts(&comp_or_interp_options);
					}
				},
				"CC" | "RUSTC" | "SH" | "SHELL" | "COMPILER" | "INTERPRETER" =>
				{
					if comp_or_interp.is_some()
					{
						return Err(syn::Error::new_spanned(
							&ident,
							format!(
								"the {} parameter must either be supplied only once or never \
								 (defaulted)",
								ident.to_string()
							)
						));
					}
					let buf;
					get_parenthesized_raw!(buf in input);
					comp_or_interp = Some(buf.to_string());
					if found_lang
					{
						language
							.set_compiler(comp_or_interp.as_ref())
							.set_shell(comp_or_interp.as_ref());
					}
				},
				"ARGS" =>
				{
					let buf;
					get_parenthesized_raw!(buf in input);
					exe_args.extend(
						buf.to_string()
							.split_whitespace()
							.map(|s| s.to_string())
							.collect::<Vec<_>>()
					);
				},
				_ =>
				{
					return Err(syn::Error::new_spanned(
						&ident,
						format!("unknown parameter `{}`", ident.to_string())
					));
				}
			}
			if input.parse::<Token![,]>().is_err()
			{
				break;
			}
		}
		Ok(Self {
			lang:   language,
			args:   exe_args,
			expand: expand.unwrap_or(RunnableExpandKind::Output)
		})
	}
}

#[allow(dead_code)]
fn longest_existing_subpath(p: &path::PathBuf) -> Option<&path::Path>
{
	for sub in p.ancestors()
	{
		if sub.exists()
		{
			return Some(sub);
		}
	}
	None
}

fn make_tempfile_path(template: impl AsRef<path::Path>) -> Option<path::PathBuf>
{
	let mut buf = fs::canonicalize(env::temp_dir()).ok()?;
	let mut rand = getrandom::u64().ok()?;

	for component in template.as_ref()
	{
		let mut bytes = component.as_encoded_bytes().to_owned();
		for b in bytes.as_mut_slice()
		{
			if *b == b'X'
			{
				let ch = (rand % 16) as u8;
				let ch = if ch < 10 { b'0' + ch } else { b'a' + (ch - 10) };
				*b = ch;

				rand /= 16;
				rand = if rand > 0
				{
					rand
				}
				else
				{
					getrandom::u64().ok()?
				};
			}
		}
		buf.push(unsafe { OsStr::from_encoded_bytes_unchecked(bytes.as_slice()) });
	}

	Some(buf)
}

fn make_tempfile(template: impl AsRef<path::Path>) -> Option<(path::PathBuf, fs::File)>
{
	let path = make_tempfile_path(template)?;
	let dir = path.parent()?;
	fs::create_dir_all(dir).ok()?;
	let file = fs::File::create_new(&path).ok()?;
	Some((path, file))
}

macro_rules! which {
	//($as_str:expr) => {
	//	::pathsearch::find_executable_in_path(($as_str).into()).unwrap_or(($as_str).into())
	//};
	($($as_ident:tt)*) => {
		::pathsearch::find_executable_in_path(stringify!($($as_ident)*).into())
			.unwrap_or(stringify!($($as_ident)*).into())
	};
}

impl RunnableItem
{
	fn get_item_decl_start(&self) -> TokenStream
	{
		match self
		{
			Self::Const(ItemConstRunnable {
				attrs,
				vis,
				const_token,
				ident,
				generics,
				ty,
				..
			}) =>
			{
				quote! {
					#(#attrs)* #vis #const_token #ident #generics: #ty
				}
			},
			Self::Static(ItemStaticRunnable {
				attrs,
				vis,
				static_token,
				mutability,
				ident,
				ty,
				..
			}) =>
			{
				quote! {
					#(#attrs)* #vis #static_token #mutability #ident: #ty
				}
			}
		}
	}

	fn get_code(&self) -> &TokenStream
	{
		match self
		{
			Self::Const(ItemConstRunnable { code, .. }) => code,
			Self::Static(ItemStaticRunnable { code, .. }) => code
		}
	}

	fn execute_clike_code(
		&self,
		mut srcfile: fs::File,
		srcfile_path: &path::Path,
		outfile_path: &path::Path,
		compiler: &String,
		std: &String,
		cc_opts: &Vec<String>,
		args: &Vec<String>
	) -> Result<process::Output, TokenStream>
	{
		if let Err(ioerr) = srcfile.write_all(self.get_code().to_string().as_bytes())
		{
			return Err(syn::Error::new(
				Span::call_site(),
				format!("unable to write to temporary source file: {}", ioerr)
			)
			.to_compile_error());
		}
		if let Err(ioerr) = srcfile.flush().and_then(|_| srcfile.sync_all())
		{
			return Err(syn::Error::new(
				Span::call_site(),
				format!("unable to write to temporary source file: {}", ioerr)
			)
			.to_compile_error());
		}
		mem::drop(srcfile);
		let output = Command::new(compiler)
			.args([
				format!("-std={std}").as_ref(),
				srcfile_path.as_os_str(),
				"-o".as_ref(),
				outfile_path.as_os_str()
			])
			.args(cc_opts)
			.output();
		if let Err(error) = output
		{
			return Err(syn::Error::new(
				Span::call_site(),
				format!("unable to execute compiler: {}", error)
			)
			.to_compile_error());
		}
		let output = output.unwrap();
		if !output.status.success()
		{
			return Err(syn::Error::new(
				Span::call_site(),
				format!(
					"compilation failed:\n{}",
					String::from_utf8_lossy(&output.stderr)
				)
			)
			.to_compile_error());
		}
		Ok(Command::new(outfile_path.as_os_str())
			.args(args)
			.output()
			.map_err(|err| {
				syn::Error::new(Span::call_site(), format!("couldn't spawn process: {err}"))
					.to_compile_error()
			})?)
	}

	fn execute_shell(
		&self,
		mut srcfile: fs::File,
		srcfile_path: &path::Path,
		shell: &String,
		shell_opts: &Vec<String>,
		args: &Vec<String>
	) -> Result<process::Output, TokenStream>
	{
		let env = which!(env);
		let header = format!(
			"#!{} -S {shell} {}\n\n",
			env.to_string_lossy(),
			shell_opts.join(" ")
		);
		let code = self.get_code().to_string();
		let mut to_write = [
			IoSlice::new(header.as_bytes()),
			IoSlice::new(code.as_bytes())
		];
		if let Err(ioerr) = srcfile.write_all_vectored(&mut to_write)
		{
			return Err(syn::Error::new(
				Span::call_site(),
				format!("unable to write to temporary source file: {}", ioerr)
			)
			.to_compile_error());
		}
		if let Err(ioerr) = srcfile.flush().and_then(|_| srcfile.sync_all())
		{
			return Err(syn::Error::new(
				Span::call_site(),
				format!("unable to write to temporary source file: {}", ioerr)
			)
			.to_compile_error());
		}
		let mut perms = srcfile.metadata().unwrap().permissions();
		perms.set_mode(perms.mode() | 0o100);
		srcfile.set_permissions(perms).unwrap();
		mem::drop(srcfile);
		Ok(Command::new(srcfile_path.as_os_str())
			.args(args)
			.output()
			.map_err(|err| {
				syn::Error::new(Span::call_site(), format!("couldn't spawn process: {err}"))
					.to_compile_error()
			})?)
	}

	fn execute_rust_code(
		&self,
		mut srcfile: fs::File,
		srcfile_path: &path::Path,
		outfile_path: &path::Path,
		compiler: &String,
		rustc_opts: &Vec<String>,
		args: &Vec<String>
	) -> Result<process::Output, TokenStream>
	{
		if let Err(ioerr) = srcfile.write_all(self.get_code().to_string().as_bytes())
		{
			return Err(syn::Error::new(
				Span::call_site(),
				format!("unable to write to temporary source file: {}", ioerr)
			)
			.to_compile_error());
		}
		if let Err(ioerr) = srcfile.flush().and_then(|_| srcfile.sync_all())
		{
			return Err(syn::Error::new(
				Span::call_site(),
				format!("unable to write to temporary source file: {}", ioerr)
			)
			.to_compile_error());
		}
		mem::drop(srcfile);
		let output = Command::new(compiler)
			.args([
				srcfile_path.as_os_str(),
				"-o".as_ref(),
				outfile_path.as_os_str()
			])
			.args(rustc_opts)
			.output();
		if let Err(error) = output
		{
			return Err(syn::Error::new(
				Span::call_site(),
				format!("unable to execute compiler: {}", error)
			)
			.to_compile_error());
		}
		let output = output.unwrap();
		if !output.status.success()
		{
			return Err(syn::Error::new(
				Span::call_site(),
				format!(
					"compilation failed:\n{}",
					String::from_utf8_lossy(&output.stderr)
				)
			)
			.to_compile_error());
		}
		Ok(Command::new(outfile_path.as_os_str())
			.args(args)
			.output()
			.map_err(|err| {
				syn::Error::new(Span::call_site(), format!("couldn't spawn process: {err}"))
					.to_compile_error()
			})?)
	}

	fn execute_code(&self, exec_info: &RunnableArgs) -> TokenStream
	{
		let &RunnableArgs { lang, args, expand } = &exec_info;
		let (srcfile_path, srcfile) = match make_tempfile(format!(
			"proc-macro-utils/generated/runnable-XXXXXXXX.{}",
			lang.format_extension()
		))
		{
			Some(okres) => okres,
			_ =>
			{
				return syn::Error::new(
					Span::call_site(),
					"unable to create temporary source file !"
				)
				.to_compile_error();
			}
		};
		let outfile_path = match make_tempfile_path("proc-macro-utils/generated/runnable-XXXXXXXX")
		{
			Some(okres) => okres,
			_ =>
			{
				return syn::Error::new(
					Span::call_site(),
					"unable to create temporary source file !"
				)
				.to_compile_error();
			}
		};
		let process_output = match lang
		{
			&RunnableLang::C {
				ref compiler,
				ref std,
				ref cc_opts
			} =>
			{
				self.execute_clike_code(
					srcfile,
					&srcfile_path,
					&outfile_path,
					compiler,
					std,
					cc_opts,
					args
				)
			},
			&RunnableLang::Cxx {
				ref compiler,
				ref std,
				ref cc_opts
			} =>
			{
				self.execute_clike_code(
					srcfile,
					&srcfile_path,
					&outfile_path,
					compiler,
					std,
					cc_opts,
					args
				)
			},
			&RunnableLang::Rust {
				ref compiler,
				ref rustc_opts
			} =>
			{
				self.execute_rust_code(
					srcfile,
					&srcfile_path,
					&outfile_path,
					compiler,
					rustc_opts,
					args
				)
			},
			&RunnableLang::Shell {
				ref shell,
				ref shell_opts
			} => self.execute_shell(srcfile, &srcfile_path, shell, shell_opts, args)
		};

		if let Err(err) = process_output.clone().and_then(|output| {
			output.status.code().ok_or(
				syn::Error::new(Span::call_site(), "the process didn't terminate normally")
					.to_compile_error()
			)
		})
		{
			return err;
		}

		let mut process_output = process_output.unwrap();

		match expand
		{
			RunnableExpandKind::ExitCode =>
			{
				let exit_code = process_output.status.code();
				quote! { #exit_code }
			},
			RunnableExpandKind::Output =>
			{
				let mut output = process_output.stderr;
				output.append(&mut process_output.stdout);
				let output = String::from_utf8_lossy(&output);
				quote! { #output }
			},
			RunnableExpandKind::StdOut =>
			{
				let stdout = String::from_utf8_lossy(&process_output.stdout);
				quote! { #stdout }
			},
			RunnableExpandKind::StdErr =>
			{
				let stderr = String::from_utf8_lossy(&process_output.stderr);
				quote! { #stderr }
			}
		}
	}
}

impl Parse for RunnableItem
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		let mut errors;

		let fork = input.fork();
		match (&fork, fork.parse())
		{
			(forked, Ok(itconst)) =>
			{
				input.advance_to(forked);
				return Ok(Self::Const(itconst));
			},
			(_, Err(err)) => errors = err
		}

		input
			.parse()
			.map_err(|err| {
				errors.combine(err);
				errors.to_owned()
			})
			.map(|it| Self::Static(it))
	}
}

/// # TODO
/// - change the needed `body! { ... }` wrapping macro to `stringify!` its args
/// - then get the raw string instead of a `TokenStream`, and see if it enables
///   us to get the input unchanged
/// - else, change the syntax to make it look like an `asm!` statement, i.e. a
///   `#[proc_macro]` instead of a `#[proc_macro_attribute]`, and with
///   `options(...)`, etc...
#[proc_macro_attribute]
pub fn runnable(input: TokenStreamClassic, annotated_item: TokenStreamClassic)
-> TokenStreamClassic
{
	let item = parse_macro_input!(annotated_item as RunnableItem);
	let args = parse_macro_input!(input as RunnableArgs);

	let decl_start = item.get_item_decl_start();
	let expansion = item.execute_code(&args);

	quote! { #decl_start = #expansion; }.into()
}

#[proc_macro_attribute]
pub fn gen_variant_names(
	_input: TokenStreamClassic,
	annotated_item: TokenStreamClassic
) -> TokenStreamClassic
{
	let item = parse_macro_input!(annotated_item as ItemEnum);

	fn mk_lower(string: String) -> String
	{
		string.to_lowercase()
	}
	fn mk_upper(string: String) -> String
	{
		string.to_uppercase()
	}
	fn mk_upper_camel(string: String) -> String
	{
		string.to_upper_camel_case()
	}
	fn mk_lower_camel(string: String) -> String
	{
		string.to_lower_camel_case()
	}
	fn mk_snake(string: String) -> String
	{
		string.to_snake_case()
	}
	fn mk_kebab(string: String) -> String
	{
		string.to_kebab_case()
	}
	fn mk_shouty_snake(string: String) -> String
	{
		string.to_shouty_snake_case()
	}
	fn mk_title(string: String) -> String
	{
		string.to_title_case()
	}
	fn mk_shouty_kebab(string: String) -> String
	{
		string.to_shouty_kebab_case()
	}
	fn mk_train(string: String) -> String
	{
		string.to_train_case()
	}

	let styled_string_constructors = [
		mk_lower as fn(String) -> String,
		mk_upper as fn(String) -> String,
		mk_upper_camel as fn(String) -> String,
		mk_lower_camel as fn(String) -> String,
		mk_snake as fn(String) -> String,
		mk_kebab as fn(String) -> String,
		mk_shouty_snake as fn(String) -> String,
		mk_title as fn(String) -> String,
		mk_shouty_kebab as fn(String) -> String,
		mk_train as fn(String) -> String
	];
	let mut variant_names_list = vec![];
	let mut variant_matchers_list = vec![];
	for variant in item.variants.clone()
	{
		let variant_ident = variant.ident.clone();
		let variant_name = variant.ident.clone().to_string();
		match variant.fields
		{
			Fields::Named(..) =>
			{
				variant_matchers_list.push(quote! {
					Self::#variant_ident { .. }
				})
			},
			Fields::Unnamed(..) =>
			{
				variant_matchers_list.push(quote! {
					Self::#variant_ident ( .. )
				})
			},
			Fields::Unit =>
			{
				variant_matchers_list.push(quote! {
					Self::#variant_ident
				})
			},
		}
		let mut styled_strings = vec![];
		for get_styled in styled_string_constructors
		{
			styled_strings.push(get_styled(variant_name.clone()));
		}
		variant_names_list.push(quote! {
			<MultiCaseStaticString>::new([ #(#styled_strings,)* ])
		});
	}

	let enum_name = item.ident.clone();
	let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

	let variant_count = item.variants.len();

	let variant_index = 0..variant_count;

	let returned_impl = quote! {
		impl #impl_generics #enum_name #ty_generics #where_clause
		{
			const VARIANT_NAMES_GENERATED: [MultiCaseStaticString; #variant_count] = [
				#(#variant_names_list,)*
			];

			pub const fn variant_name(&self, style: CaseKind) -> &'static str
			{
				match self
				{
					#(#variant_matchers_list => Self::VARIANT_NAMES_GENERATED[#variant_index].get(style),)*
					_ => unreachable!()
				}
			}
		}
	};
	quote! { #item #returned_impl }.into()
}

//#[proc_macro_attribute]
// pub fn embpp(input: TokenStreamClassic) -> TokenStreamClassic
//{
// 	todo!()
//}
