use proc_macro_error2::ResultExt as _;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{ForeignItem, ImplItem, Item, TraitItem};

use crate::imp::{GetAttrs as _, WithoutAttrs as _, item::parse::UnsafeItemKind};

mod parse;

#[derive(Debug)]
struct UnsafeItemSafety
{
	parsed: parse::UnsafeItemAttr
}

impl UnsafeItemSafety
{
	fn new(attr: TokenStream2) -> syn::Result<(Self, UnsafeItemKind)>
	{
		let this = Self {
			parsed: syn::parse2(attr)?
		};
		let kind = (&this.parsed).into();
		Ok((this, kind))
	}
}

impl ToTokens for UnsafeItemSafety
{
	fn to_tokens(&self, tokens: &mut TokenStream2)
	{
		let lines = match &self.parsed
		{
			parse::UnsafeItemAttr::SafetyOnly { lines }
			| parse::UnsafeItemAttr::Detailled {
				safety: parse::UnsafeItemAttrSafety { lines, .. },
				..
			} => lines.iter().collect::<Vec<_>>()
		};
		let (mut pre, mut post, mut invariant) = (vec![], vec![], vec![]);
		if let parse::UnsafeItemAttr::Detailled {
			pre: parsed_pre,
			post: parsed_post,
			invariant: parsed_invariant,
			..
		} = &self.parsed
		{
			pre = parsed_pre.clone();
			post = parsed_post.clone();
			invariant = parsed_invariant.clone();
		}
		tokens.extend(quote! {
			#[doc = ""]
			#[doc = "# Safety"]
			#[doc = ""]
			#(#[doc = #lines])*
			#(#[::contracts::debug_requires(#pre)])*
			#(#[::contracts::debug_ensures(#post)])*
			#(#[::contracts::debug_invariant(#invariant)])*
			#[expect(unsafe_code)]
		});
	}
}

enum UnsafeItemInner
{
	Bare(Item),
	InImpl(ImplItem),
	InTrait(TraitItem),
	InExtern(ForeignItem)
}

struct UnsafeItem
{
	safety: UnsafeItemSafety,
	item:   UnsafeItemInner
}

impl UnsafeItem
{
	fn new(attr: TokenStream2, item: TokenStream2) -> syn::Result<Self>
	{
		UnsafeItemSafety::new(attr).and_then(|(safety, kind)| {
			Ok(match kind
			{
				UnsafeItemKind::Bare =>
				{
					Self {
						safety,
						item: UnsafeItemInner::Bare(syn::parse2(item)?)
					}
				},
				UnsafeItemKind::InImpl =>
				{
					Self {
						safety,
						item: UnsafeItemInner::InImpl(syn::parse2(item)?)
					}
				},
				UnsafeItemKind::InTrait =>
				{
					Self {
						safety,
						item: UnsafeItemInner::InTrait(syn::parse2(item)?)
					}
				},
				UnsafeItemKind::InExtern =>
				{
					Self {
						safety,
						item: UnsafeItemInner::InExtern(syn::parse2(item)?)
					}
				},
			})
		})
	}
}

impl ToTokens for UnsafeItem
{
	fn to_tokens(&self, tokens: &mut TokenStream2)
	{
		match self
		{
			Self {
				safety,
				item: UnsafeItemInner::Bare(item)
			} =>
			{
				let item_attrs = item.attrs().cloned().unwrap_or_default();
				let without_attrs = item.expand_no_attrs();
				quote! {
					#(#item_attrs)*
					#safety
					#without_attrs
				}
			},
			Self {
				safety,
				item: UnsafeItemInner::InImpl(item)
			} =>
			{
				let item_attrs = item.attrs().cloned().unwrap_or_default();
				let without_attrs = item.expand_no_attrs();
				quote! {
					#(#item_attrs)*
					#safety
					#without_attrs
				}
			},
			Self {
				safety,
				item: UnsafeItemInner::InTrait(item)
			} =>
			{
				let item_attrs = item.attrs().cloned().unwrap_or_default();
				let without_attrs = item.expand_no_attrs();
				quote! {
					#(#item_attrs)*
					#safety
					#without_attrs
				}
			},
			Self {
				safety,
				item: UnsafeItemInner::InExtern(item)
			} =>
			{
				let item_attrs = item.attrs().cloned().unwrap_or_default();
				let without_attrs = item.expand_no_attrs();
				quote! {
					#(#item_attrs)*
					#safety
					#without_attrs
				}
			}
		}
		.to_tokens(tokens);
	}
}

pub(crate) fn item(attr: TokenStream2, item: TokenStream2) -> TokenStream2
{
	UnsafeItem::new(attr, item)
		.unwrap_or_abort()
		.into_token_stream()
}
