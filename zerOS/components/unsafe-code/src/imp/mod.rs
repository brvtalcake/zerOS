use std::any::type_name_of_val;

use proc_macro::Ident;
use proc_macro_error2::emit_warning;
use proc_macro2::{Ident as Ident2, Span as Span2, TokenStream as TokenStream2};
use quote::ToTokens;
use syn::{
	Attribute,
	Expr,
	ExprArray,
	ExprAssign,
	ExprAsync,
	ExprAwait,
	ExprBinary,
	ExprBlock,
	ExprBreak,
	ExprCall,
	ExprCast,
	ExprClosure,
	ExprConst,
	ExprContinue,
	ExprField,
	ExprForLoop,
	ExprGroup,
	ExprIf,
	ExprIndex,
	ExprInfer,
	ExprLet,
	ExprLit,
	ExprLoop,
	ExprMacro,
	ExprMatch,
	ExprMethodCall,
	ExprParen,
	ExprPath,
	ExprRange,
	ExprRawAddr,
	ExprReference,
	ExprRepeat,
	ExprReturn,
	ExprStruct,
	ExprTry,
	ExprTryBlock,
	ExprTuple,
	ExprUnary,
	ExprUnsafe,
	ExprWhile,
	ExprYield,
	ForeignItem,
	ForeignItemFn,
	ForeignItemMacro,
	ForeignItemStatic,
	ForeignItemType,
	ImplItem,
	ImplItemConst,
	ImplItemFn,
	ImplItemMacro,
	ImplItemType,
	Item,
	ItemConst,
	ItemEnum,
	ItemExternCrate,
	ItemFn,
	ItemForeignMod,
	ItemImpl,
	ItemMacro,
	ItemMod,
	ItemStatic,
	ItemStruct,
	ItemTrait,
	ItemTraitAlias,
	ItemType,
	ItemUnion,
	ItemUse,
	Lit,
	Local,
	Stmt,
	StmtMacro,
	TraitItem,
	TraitItemConst,
	TraitItemFn,
	TraitItemMacro,
	TraitItemType,
	parse::{Parse, ParseStream},
	spanned::Spanned
};
use unicode_xid::UnicodeXID as _;

mod block;
pub(crate) use block::block;
mod item;
pub(crate) use item::item;

struct InnerSafetyDocAttr
{
	pub(crate) safety: Vec<String>,
	pub(crate) others: Vec<Attribute>
}

impl Parse for InnerSafetyDocAttr
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		let mut safety = vec![];
		let mut others = vec![];

		let attrs = input.call(Attribute::parse_inner)?;
		for attr in attrs
		{
			let path = attr.path();
			if path.require_ident()?.to_string().as_str() == "doc"
			{
				safety.push(match &attr.meta.require_name_value()?.value
				{
					Expr::Lit(ExprLit {
						lit: Lit::Str(s), ..
					}) => Ok(s.value()),
					other => Err(syn::Error::new_spanned(other, "expected a string literal"))
				}?);
			}
			else
			{
				others.push(attr);
			}
		}
		Ok(Self { safety, others })
	}
}

trait GetAttrs
{
	fn attrs(&self) -> Option<&Vec<Attribute>>;
	fn attrs_mut(&mut self) -> Option<&mut Vec<Attribute>>;
}

impl GetAttrs for Expr
{
	fn attrs(&self) -> Option<&Vec<Attribute>>
	{
		Some(match self
		{
			Self::Array(ExprArray { attrs, .. })
			| Self::Assign(ExprAssign { attrs, .. })
			| Self::Async(ExprAsync { attrs, .. })
			| Self::Await(ExprAwait { attrs, .. })
			| Self::Binary(ExprBinary { attrs, .. })
			| Self::Block(ExprBlock { attrs, .. })
			| Self::Break(ExprBreak { attrs, .. })
			| Self::Call(ExprCall { attrs, .. })
			| Self::Cast(ExprCast { attrs, .. })
			| Self::Closure(ExprClosure { attrs, .. })
			| Self::Const(ExprConst { attrs, .. })
			| Self::Continue(ExprContinue { attrs, .. })
			| Self::Field(ExprField { attrs, .. })
			| Self::ForLoop(ExprForLoop { attrs, .. })
			| Self::Group(ExprGroup { attrs, .. })
			| Self::If(ExprIf { attrs, .. })
			| Self::Index(ExprIndex { attrs, .. })
			| Self::Infer(ExprInfer { attrs, .. })
			| Self::Let(ExprLet { attrs, .. })
			| Self::Lit(ExprLit { attrs, .. })
			| Self::Loop(ExprLoop { attrs, .. })
			| Self::Macro(ExprMacro { attrs, .. })
			| Self::Match(ExprMatch { attrs, .. })
			| Self::MethodCall(ExprMethodCall { attrs, .. })
			| Self::Paren(ExprParen { attrs, .. })
			| Self::Path(ExprPath { attrs, .. })
			| Self::Range(ExprRange { attrs, .. })
			| Self::RawAddr(ExprRawAddr { attrs, .. })
			| Self::Reference(ExprReference { attrs, .. })
			| Self::Repeat(ExprRepeat { attrs, .. })
			| Self::Return(ExprReturn { attrs, .. })
			| Self::Struct(ExprStruct { attrs, .. })
			| Self::Try(ExprTry { attrs, .. })
			| Self::TryBlock(ExprTryBlock { attrs, .. })
			| Self::Tuple(ExprTuple { attrs, .. })
			| Self::Unary(ExprUnary { attrs, .. })
			| Self::Unsafe(ExprUnsafe { attrs, .. })
			| Self::While(ExprWhile { attrs, .. })
			| Self::Yield(ExprYield { attrs, .. }) => attrs,
			Self::Verbatim(_) => None?,
			unhandled =>
			{
				emit_warning!(
					self,
					"the code at {}:{}:{} needs refactoring",
					file!(),
					line!(),
					column!();
					note = "a variant has been added to `{}`, which could not be handled", type_name_of_val(self);
					help = "its debug representation is: {:?}", unhandled;
				);
				None?
			}
		})
	}

	fn attrs_mut(&mut self) -> Option<&mut Vec<Attribute>>
	{
		Some(match self
		{
			Self::Array(ExprArray { attrs, .. })
			| Self::Assign(ExprAssign { attrs, .. })
			| Self::Async(ExprAsync { attrs, .. })
			| Self::Await(ExprAwait { attrs, .. })
			| Self::Binary(ExprBinary { attrs, .. })
			| Self::Block(ExprBlock { attrs, .. })
			| Self::Break(ExprBreak { attrs, .. })
			| Self::Call(ExprCall { attrs, .. })
			| Self::Cast(ExprCast { attrs, .. })
			| Self::Closure(ExprClosure { attrs, .. })
			| Self::Const(ExprConst { attrs, .. })
			| Self::Continue(ExprContinue { attrs, .. })
			| Self::Field(ExprField { attrs, .. })
			| Self::ForLoop(ExprForLoop { attrs, .. })
			| Self::Group(ExprGroup { attrs, .. })
			| Self::If(ExprIf { attrs, .. })
			| Self::Index(ExprIndex { attrs, .. })
			| Self::Infer(ExprInfer { attrs, .. })
			| Self::Let(ExprLet { attrs, .. })
			| Self::Lit(ExprLit { attrs, .. })
			| Self::Loop(ExprLoop { attrs, .. })
			| Self::Macro(ExprMacro { attrs, .. })
			| Self::Match(ExprMatch { attrs, .. })
			| Self::MethodCall(ExprMethodCall { attrs, .. })
			| Self::Paren(ExprParen { attrs, .. })
			| Self::Path(ExprPath { attrs, .. })
			| Self::Range(ExprRange { attrs, .. })
			| Self::RawAddr(ExprRawAddr { attrs, .. })
			| Self::Reference(ExprReference { attrs, .. })
			| Self::Repeat(ExprRepeat { attrs, .. })
			| Self::Return(ExprReturn { attrs, .. })
			| Self::Struct(ExprStruct { attrs, .. })
			| Self::Try(ExprTry { attrs, .. })
			| Self::TryBlock(ExprTryBlock { attrs, .. })
			| Self::Tuple(ExprTuple { attrs, .. })
			| Self::Unary(ExprUnary { attrs, .. })
			| Self::Unsafe(ExprUnsafe { attrs, .. })
			| Self::While(ExprWhile { attrs, .. })
			| Self::Yield(ExprYield { attrs, .. }) => attrs,
			Self::Verbatim(_) => None?,
			ref unhandled =>
			{
				emit_warning!(
					self,
					"the code at {}:{}:{} needs refactoring",
					file!(),
					line!(),
					column!();
					note = "a variant has been added to `{}`, which could not be handled", type_name_of_val(self);
					help = "its debug representation is: {:?}", unhandled;
				);
				None?
			}
		})
	}
}

impl GetAttrs for Item
{
	fn attrs(&self) -> Option<&Vec<Attribute>>
	{
		Some(match self
		{
			Self::Const(ItemConst { attrs, .. })
			| Self::Enum(ItemEnum { attrs, .. })
			| Self::ExternCrate(ItemExternCrate { attrs, .. })
			| Self::Fn(ItemFn { attrs, .. })
			| Self::ForeignMod(ItemForeignMod { attrs, .. })
			| Self::Impl(ItemImpl { attrs, .. })
			| Self::Macro(ItemMacro { attrs, .. })
			| Self::Mod(ItemMod { attrs, .. })
			| Self::Static(ItemStatic { attrs, .. })
			| Self::Struct(ItemStruct { attrs, .. })
			| Self::Trait(ItemTrait { attrs, .. })
			| Self::TraitAlias(ItemTraitAlias { attrs, .. })
			| Self::Type(ItemType { attrs, .. })
			| Self::Union(ItemUnion { attrs, .. })
			| Self::Use(ItemUse { attrs, .. }) => attrs,
			Self::Verbatim(_) => None?,
			unhandled =>
			{
				emit_warning!(
					self,
					"the code at {}:{}:{} needs refactoring",
					file!(),
					line!(),
					column!();
					note = "a variant has been added to `{}`, which could not be handled", type_name_of_val(self);
					help = "its debug representation is: {:?}", unhandled;
				);
				None?
			}
		})
	}

	fn attrs_mut(&mut self) -> Option<&mut Vec<Attribute>>
	{
		Some(match self
		{
			Self::Const(ItemConst { attrs, .. })
			| Self::Enum(ItemEnum { attrs, .. })
			| Self::ExternCrate(ItemExternCrate { attrs, .. })
			| Self::Fn(ItemFn { attrs, .. })
			| Self::ForeignMod(ItemForeignMod { attrs, .. })
			| Self::Impl(ItemImpl { attrs, .. })
			| Self::Macro(ItemMacro { attrs, .. })
			| Self::Mod(ItemMod { attrs, .. })
			| Self::Static(ItemStatic { attrs, .. })
			| Self::Struct(ItemStruct { attrs, .. })
			| Self::Trait(ItemTrait { attrs, .. })
			| Self::TraitAlias(ItemTraitAlias { attrs, .. })
			| Self::Type(ItemType { attrs, .. })
			| Self::Union(ItemUnion { attrs, .. })
			| Self::Use(ItemUse { attrs, .. }) => attrs,
			Self::Verbatim(_) => None?,
			ref unhandled =>
			{
				emit_warning!(
					self,
					"the code at {}:{}:{} needs refactoring",
					file!(),
					line!(),
					column!();
					note = "a variant has been added to `{}`, which could not be handled", type_name_of_val(self);
					help = "its debug representation is: {:?}", unhandled;
				);
				None?
			}
		})
	}
}

impl GetAttrs for Local
{
	fn attrs(&self) -> Option<&Vec<Attribute>>
	{
		Some(&self.attrs)
	}

	fn attrs_mut(&mut self) -> Option<&mut Vec<Attribute>>
	{
		Some(&mut self.attrs)
	}
}

impl GetAttrs for StmtMacro
{
	fn attrs(&self) -> Option<&Vec<Attribute>>
	{
		Some(&self.attrs)
	}

	fn attrs_mut(&mut self) -> Option<&mut Vec<Attribute>>
	{
		Some(&mut self.attrs)
	}
}

impl GetAttrs for Stmt
{
	fn attrs(&self) -> Option<&Vec<Attribute>>
	{
		match self
		{
			Self::Expr(expr, ..) => expr.attrs(),
			Self::Item(item) => item.attrs(),
			Self::Local(local) => local.attrs(),
			Self::Macro(mac) => mac.attrs()
		}
	}

	fn attrs_mut(&mut self) -> Option<&mut Vec<Attribute>>
	{
		match self
		{
			Self::Expr(expr, ..) => expr.attrs_mut(),
			Self::Item(item) => item.attrs_mut(),
			Self::Local(local) => local.attrs_mut(),
			Self::Macro(mac) => mac.attrs_mut()
		}
	}
}

impl GetAttrs for ImplItem
{
	fn attrs(&self) -> Option<&Vec<Attribute>>
	{
		Some(match self
		{
			Self::Const(ImplItemConst { attrs, .. })
			| Self::Fn(ImplItemFn { attrs, .. })
			| Self::Macro(ImplItemMacro { attrs, .. })
			| Self::Type(ImplItemType { attrs, .. }) => attrs,
			Self::Verbatim(_) => None?,
			unhandled =>
			{
				emit_warning!(
					self,
					"the code at {}:{}:{} needs refactoring",
					file!(),
					line!(),
					column!();
					note = "a variant has been added to `{}`, which could not be handled", type_name_of_val(self);
					help = "its debug representation is: {:?}", unhandled;
				);
				None?
			}
		})
	}

	fn attrs_mut(&mut self) -> Option<&mut Vec<Attribute>>
	{
		Some(match self
		{
			Self::Const(ImplItemConst { attrs, .. })
			| Self::Fn(ImplItemFn { attrs, .. })
			| Self::Macro(ImplItemMacro { attrs, .. })
			| Self::Type(ImplItemType { attrs, .. }) => attrs,
			Self::Verbatim(_) => None?,
			ref unhandled =>
			{
				emit_warning!(
					self,
					"the code at {}:{}:{} needs refactoring",
					file!(),
					line!(),
					column!();
					note = "a variant has been added to `{}`, which could not be handled", type_name_of_val(self);
					help = "its debug representation is: {:?}", unhandled;
				);
				None?
			}
		})
	}
}

impl GetAttrs for TraitItem
{
	fn attrs(&self) -> Option<&Vec<Attribute>>
	{
		Some(match self
		{
			Self::Const(TraitItemConst { attrs, .. })
			| Self::Fn(TraitItemFn { attrs, .. })
			| Self::Macro(TraitItemMacro { attrs, .. })
			| Self::Type(TraitItemType { attrs, .. }) => attrs,
			Self::Verbatim(_) => None?,
			unhandled =>
			{
				emit_warning!(
					self,
					"the code at {}:{}:{} needs refactoring",
					file!(),
					line!(),
					column!();
					note = "a variant has been added to `{}`, which could not be handled", type_name_of_val(self);
					help = "its debug representation is: {:?}", unhandled;
				);
				None?
			}
		})
	}

	fn attrs_mut(&mut self) -> Option<&mut Vec<Attribute>>
	{
		Some(match self
		{
			Self::Const(TraitItemConst { attrs, .. })
			| Self::Fn(TraitItemFn { attrs, .. })
			| Self::Macro(TraitItemMacro { attrs, .. })
			| Self::Type(TraitItemType { attrs, .. }) => attrs,
			Self::Verbatim(_) => None?,
			ref unhandled =>
			{
				emit_warning!(
					self,
					"the code at {}:{}:{} needs refactoring",
					file!(),
					line!(),
					column!();
					note = "a variant has been added to `{}`, which could not be handled", type_name_of_val(self);
					help = "its debug representation is: {:?}", unhandled;
				);
				None?
			}
		})
	}
}

impl GetAttrs for ForeignItem
{
	fn attrs(&self) -> Option<&Vec<Attribute>>
	{
		Some(match self
		{
			Self::Static(ForeignItemStatic { attrs, .. })
			| Self::Fn(ForeignItemFn { attrs, .. })
			| Self::Macro(ForeignItemMacro { attrs, .. })
			| Self::Type(ForeignItemType { attrs, .. }) => attrs,
			Self::Verbatim(_) => None?,
			unhandled =>
			{
				emit_warning!(
					self,
					"the code at {}:{}:{} needs refactoring",
					file!(),
					line!(),
					column!();
					note = "a variant has been added to `{}`, which could not be handled", type_name_of_val(self);
					help = "its debug representation is: {:?}", unhandled;
				);
				None?
			}
		})
	}

	fn attrs_mut(&mut self) -> Option<&mut Vec<Attribute>>
	{
		Some(match self
		{
			Self::Static(ForeignItemStatic { attrs, .. })
			| Self::Fn(ForeignItemFn { attrs, .. })
			| Self::Macro(ForeignItemMacro { attrs, .. })
			| Self::Type(ForeignItemType { attrs, .. }) => attrs,
			Self::Verbatim(_) => None?,
			ref unhandled =>
			{
				emit_warning!(
					self,
					"the code at {}:{}:{} needs refactoring",
					file!(),
					line!(),
					column!();
					note = "a variant has been added to `{}`, which could not be handled", type_name_of_val(self);
					help = "its debug representation is: {:?}", unhandled;
				);
				None?
			}
		})
	}
}

trait IdentCapable
{
	fn as_ident_name(&self) -> String;
}

fn make_char_ident_continue_capable(ch: char) -> char
{
	if ch.is_xid_continue() { ch } else { '_' }
}

impl IdentCapable for str
{
	fn as_ident_name(&self) -> String
	{
		let mut iter = self.chars();
		let first = if let Some(ch) = iter.next()
		{
			if ch.is_xid_start() { ch } else { '_' }
		}
		else
		{
			return "".into();
		};
		[first]
			.into_iter()
			.chain(iter.map(make_char_ident_continue_capable))
			.collect::<String>()
	}
}

impl IdentCapable for String
{
	fn as_ident_name(&self) -> String
	{
		self.as_str().as_ident_name()
	}
}

impl IdentCapable for Ident
{
	fn as_ident_name(&self) -> String
	{
		self.to_string()
	}
}

impl IdentCapable for Ident2
{
	fn as_ident_name(&self) -> String
	{
		self.to_string()
	}
}

impl<T: IdentCapable> IdentCapable for &T
{
	fn as_ident_name(&self) -> String
	{
		(*self).as_ident_name()
	}
}

trait JoinSpans
{
	fn joined_span(&self) -> Span2;
}

impl<T: ?Sized> JoinSpans for T
where
	for<'a> &'a T: IntoIterator<Item: Spanned>
{
	fn joined_span(&self) -> Span2
	{
		let mut iter = self.into_iter();
		let mut span = iter
			.next()
			.as_ref()
			.map(Spanned::span)
			.unwrap_or_else(Span2::call_site);

		for item in iter
		{
			span = span.join(item.span()).unwrap();
		}

		span
	}
}

trait WithoutAttrs
{
	fn expand_no_attrs(&self) -> TokenStream2;
}

impl<T: GetAttrs + ToTokens + Clone> WithoutAttrs for T
{
	fn expand_no_attrs(&self) -> TokenStream2
	{
		let mut cloned = self.clone();
		cloned.attrs_mut().map(|vec| vec.clear());
		cloned.into_token_stream()
	}
}
