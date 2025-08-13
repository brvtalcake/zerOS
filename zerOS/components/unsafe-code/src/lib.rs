#![allow(non_snake_case)]
#![feature(result_option_map_or_default)]
#![feature(iterator_try_collect)]
#![feature(min_specialization)]
#![feature(proc_macro_span)]

use eager2::eager_proc_macro;
use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;

mod imp;

/// # Example
/// ```rust
/// #![feature(stmt_expr_attributes)]
/// # fn main() {
/// let option = Some(5);
/// let unwrapped = zerOS_unsafe::block! {
/// 	//! SAFETY: this is safe because of ...
/// 	#[pre(option.is_some())]
/// 	#[post(tmp == 5)]
/// 	let tmp = option.unwrap_unchecked();
/// 	tmp
/// };
/// # }
/// ```
#[eager_proc_macro]
#[proc_macro_error]
pub fn block(input: TokenStream) -> TokenStream
{
	crate::imp::block(input.into()).into()
}

/// # Example
///
/// ## Freestanding function (/ item)
///
/// ```rust,compile_fail
/// /// # Example
/// /// <doc-comments>...
/// #[zerOS_unsafe::item(
/// 	into(bare),
/// 	safety(
/// 		"The caller must ensure that ...",
/// 		"... another safety documentation line ...",
/// 		include("shared-documentation.md")
/// 	),
/// 	pre(arg.is_some()), // is repeatable multiple times
/// 	post(ret.is_ok()),	// same for this one
/// 	// invariant(<invariant>...)
/// )]
/// // or, alternatively (since `bare` is the default):
/// // #[zerOS_unsafe::item(
/// // 	"The caller must ensure that ...",
/// // 	"... another safety documentation line ..."
/// // )]
/// unsafe fn function<T: Default + TryInto<usize>>(
/// 	arg: Option<T>
/// ) -> Result<usize, <T as TryInto<usize>>::Error>
/// {
/// 	arg.unwrap_or_default().try_into()
/// }
/// # fn main() {}
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn item(attr: TokenStream, item: TokenStream) -> TokenStream
{
	crate::imp::item(attr.into(), item.into()).into()
}
