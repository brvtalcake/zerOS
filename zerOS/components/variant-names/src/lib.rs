#![allow(non_snake_case)]

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
use proc_macro::TokenStream;
use quote::quote;
use syn::{Fields, ItemEnum, parse_macro_input};

#[proc_macro_attribute]
pub fn gen_variant_names(_input: TokenStream, annotated_item: TokenStream) -> TokenStream
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
