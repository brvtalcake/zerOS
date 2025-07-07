#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(decl_macro)]
#![recursion_limit = "1024"]

use eager2::eager_macro;
pub use eager2::{eager, lazy};

#[macro_export]
#[eager_macro]
macro_rules! concat_idents {
    ($($e:ident),+ $(,)?) => {
		eager2::unstringify!(
			::eager2::concat!(
        		$(
					::eager2::stringify!($e)
				),*
			)
		)
	};
}

#[macro_export]
#[doc(hidden)]
#[eager_macro]
macro_rules! __ctor_impl {
    (
        @NAME_IMPL[$($name:tt)*];
        @PRIO_IMPL[$($prio:tt)*];
        @name($($new_name:tt)*);
        $($rest:tt)*
    ) => {
        __ctor_impl!{
            @NAME_IMPL[$($new_name)*];
            @PRIO_IMPL[$($prio)*];
            $($rest)*
        }
    };

    (
        @NAME_IMPL[$($name:tt)*];
        @PRIO_IMPL[$($prio:tt)*];
        @priority($($new_prio:tt)*);
        $($rest:tt)*
    ) => {
        __ctor_impl!{
            @NAME_IMPL[$($name)*];
            @PRIO_IMPL[$($new_prio)*];
            $($rest)*
        }
    };

    (
        @NAME_IMPL[$name:ident];
        @PRIO_IMPL[$prio:literal];
        $($body:stmt)*
    ) => {
        ::eager2::eager! {
            mod concat_idents!($name, _generated_module)
            {
                use super::*;

                #[unsafe(link_section = ".bootcode")]
                unsafe extern "C" fn concat_idents!($name, _generated_function) ()
                {
                    ::eager2::lazy! {
                        $($body)*
                    }
                }

                #[unsafe(link_section = ::eager2::concat!(".ctors_init_array.", ::eager2::stringify!($prio)))]
                #[used(linker)]
                #[allow(non_upper_case_globals)]
                static concat_idents!($name, _generated_ctor): crate::init::ctors::Ctor = concat_idents!($name, _generated_function);
            }
        }
    };
}

#[macro_export]
#[eager_macro]
macro_rules! ctor {
    ($($tokens:tt)*) => {
        __ctor_impl! {
            @NAME_IMPL[];
            @PRIO_IMPL[];
            $($tokens)*
        }
    };
}

/// Stolen from `core::intrinsics::const_eval_select!`.
///
/// A macro to make it easier to invoke `const_eval_select`. Use as follows:
/// ```rust,ignore (just a macro example)
/// const_eval_select!(
///     @capture { arg1: i32 = some_expr, arg2: T = other_expr } -> U:
///     if const #[attributes_for_const_arm] {
///         // Compile-time code goes here.
///     } else #[attributes_for_runtime_arm] {
///         // Run-time code goes here.
///     }
/// )
/// ```
/// The `@capture` block declares which surrounding variables / expressions can
/// be used inside the `if const`.
/// Note that the two arms of this `if` really each become their own function,
/// which is why the macro supports setting attributes for those functions. The
/// runtime function is always markes as `#[inline]`.
///
/// See [`const_eval_select()`] for the rules and requirements around that
/// intrinsic.
pub macro const_eval_select {
    (
        @capture$([$($binders:tt)*])? { $($arg:ident : $ty:ty = $val:expr),* $(,)? } $( -> $ret:ty )? :
        if const
            $(#[$compiletime_attr:meta])* $compiletime:block
        else
            $(#[$runtime_attr:meta])* $runtime:block
    ) => {
        // Use the `noinline` arm, after adding explicit `inline` attributes
        $crate::const_eval_select!(
            @capture$([$($binders)*])? { $($arg : $ty = $val),* } $(-> $ret)? :
            #[noinline]
            if const
                #[inline] // prevent codegen on this function
                $(#[$compiletime_attr])*
                $compiletime
            else
                #[inline] // avoid the overhead of an extra fn call
                $(#[$runtime_attr])*
                $runtime
        )
    },
    // With a leading #[noinline], we don't add inline attributes
    (
        @capture$([$($binders:tt)*])? { $($arg:ident : $ty:ty = $val:expr),* $(,)? } $( -> $ret:ty )? :
        #[noinline]
        if const
            $(#[$compiletime_attr:meta])* $compiletime:block
        else
            $(#[$runtime_attr:meta])* $runtime:block
    ) => {{
        $(#[$runtime_attr])*
        fn runtime$(<$($binders)*>)?($($arg: $ty),*) $( -> $ret )? {
            $runtime
        }

        $(#[$compiletime_attr])*
        const fn compiletime$(<$($binders)*>)?($($arg: $ty),*) $( -> $ret )? {
            // Don't warn if one of the arguments is unused.
            $(let _ = $arg;)*

            $compiletime
        }

        ::core::intrinsics::const_eval_select(($($val,)*), compiletime, runtime)
    }},
    // We support leaving away the `val` expressions for *all* arguments
    // (but not for *some* arguments, that's too tricky).
    (
        @capture$([$($binders:tt)*])? { $($arg:ident : $ty:ty),* $(,)? } $( -> $ret:ty )? :
        if const
            $(#[$compiletime_attr:meta])* $compiletime:block
        else
            $(#[$runtime_attr:meta])* $runtime:block
    ) => {
        $crate::const_eval_select!(
            @capture$([$($binders)*])? { $($arg : $ty = $arg),* } $(-> $ret)? :
            if const
                $(#[$compiletime_attr])* $compiletime
            else
                $(#[$runtime_attr])* $runtime
        )
    },
}

#[macro_export]
macro_rules! callback {
    ($cb:ident( $($args:tt)* )) => {
        $cb!( $($args)* )
    };
    (( $($args:tt)* )) => {
        $($args)*
    };
    (@foreach $cb:ident( $($args:tt)* )) => {
        callback_foreach!( $cb ( $($args)* ) )
    };
    (@foreach @delim($($tokens:tt)*) $cb:ident( $($args:tt)* )) => {
        callback_foreach!( @delim($($tokens)*) $cb( $($args)* ) )
    };
}

#[macro_export]
macro_rules! callback_foreach {
    ($cb:ident( $first:tt $(,$args:tt)* )) => {
        callback_foreach!(@delim() $cb( $first $(,$args)* ))
    };
    (@delim() $cb:ident( $first:tt $(,$args:tt)* )) => {
        callback!($cb( $first ))
        $(callback!($cb( $args )))*
    };
    (@delim($delimtokens:tt) $cb:ident( $first:tt $(,$args:tt)* )) => {
        callback!($cb( $first ))
        $($delimtokens callback!($cb( $args )))*
    };
}

#[macro_export]
macro_rules! identity_expand {
    ($($tokens:tt)*) => {
        $($tokens)*
    };
    {$($tokens:tt)*} => {
        $($tokens)*
    };
    [$($tokens:tt)*] => {
        $($tokens)*
    };
}

#[macro_export]
macro_rules! count_exprs {
    ()          => { 0usize };
    ($_a:expr)  => { 1usize };
    ($first:expr $(,$rest:expr)*)
                => { 1usize + count_exprs!(@inner $(,$rest)*) };

    (@inner, $_a:expr)  => { 1usize };
    (@inner, $first:expr $(,$rest:expr)*)
        => { 1usize + count_exprs!(@inner $(,$rest)*) };
}

#[macro_export]
macro_rules! count_idents {
    ()          => { 0usize };
    ($_a:ident)  => { 1usize };
    ($first:ident $(,$rest:ident)*)
                => { 1usize + count_idents!(@inner $(,$rest)*) };

    (@inner, $_a:ident)  => { 1usize };
    (@inner, $first:ident $(,$rest:ident)*)
        => { 1usize + count_idents!(@inner $(,$rest)*) };
}

#[macro_export]
macro_rules! static_max {
    () => { 0_usize };
    ($expr:expr) => { const { ($expr) } };
    ($first:expr $(,$rest:expr)*)
        => { const {
            let recursed_max = static_max!(@inner $(,$rest)*);
            if ($first) >= (recursed_max)
            { ($first) }
            else
            { recursed_max }
         } };
    (@inner, $expr:expr) => { const { ($expr) } };
    (@inner, $first:expr $(,$rest:expr)*)
        => { const {
            let recursed_max = static_max!(@inner $(,$rest)*);
            if ($first) >= (recursed_max)
            { ($first) }
            else
            { recursed_max }
         } };
}

#[macro_export]
macro_rules! max {
    () => { 0_usize };
    ($expr:expr) => { { ($expr) } };
    ($first:expr $(,$rest:expr)*)
        => { {
            let recursed_max = max!(@inner $(,$rest)*);
            if ($first) >= (recursed_max)
            { ($first) }
            else
            { recursed_max }
         } };
    (@inner, $expr:expr) => { { ($expr) } };
    (@inner, $first:expr $(,$rest:expr)*)
        => { {
            let recursed_max = max!(@inner $(,$rest)*);
            if ($first) >= (recursed_max)
            { ($first) }
            else
            { recursed_max }
         } };
}

#[macro_export]
macro_rules! static_min {
    () => { 0_usize };
    ($expr:expr) => { const { ($expr) } };
    ($first:expr $(,$rest:expr)*)
        => { const {
            let recursed_min = static_min!(@inner $(,$rest)*);
            if ($first) <= (recursed_min)
            { ($first) }
            else
            { recursed_min }
         } };
    (@inner, $expr:expr) => { const { ($expr) } };
    (@inner, $first:expr $(,$rest:expr)*)
        => { const {
            let recursed_min = static_min!(@inner $(,$rest)*);
            if ($first) <= (recursed_min)
            { ($first) }
            else
            { recursed_min }
         } };
}

#[macro_export]
macro_rules! min {
    () => { 0_usize };
    ($expr:expr) => { { ($expr) } };
    ($first:expr $(,$rest:expr)*)
        => { {
            let recursed_min = min!(@inner $(,$rest)*);
            if ($first) <= (recursed_min)
            { ($first) }
            else
            { recursed_min }
         } };
    (@inner, $expr:expr) => { { ($expr) } };
    (@inner, $first:expr $(,$rest:expr)*)
        => { {
            let recursed_min = min!(@inner $(,$rest)*);
            if ($first) <= (recursed_min)
            { ($first) }
            else
            { recursed_min }
         } };
}

#[repr(usize)]
pub enum CaseKind
{
	LowerCase = 0,
	UpperCase,
	UpperCamelCase,
	LowerCamelCase,
	SnakeCase,
	KebabCase,
	ShoutySnakeCase,
	TitleCase,
	ShoutyKebabCase,
	TrainCase
}

pub struct MultiCaseStaticString
{
	strings: [&'static str; 10]
}

impl MultiCaseStaticString
{
	pub const fn new(strings: [&'static str; 10]) -> Self
	{
		Self { strings }
	}

	pub const fn get(&self, style: CaseKind) -> &'static str
	{
		self.strings[style as usize]
	}
}
