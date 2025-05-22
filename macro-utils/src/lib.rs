#![feature(concat_idents)]
#![feature(decl_macro)]

#![no_std]
#![recursion_limit = "1024"]

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
/// The `@capture` block declares which surrounding variables / expressions can be
/// used inside the `if const`.
/// Note that the two arms of this `if` really each become their own function, which is why the
/// macro supports setting attributes for those functions. The runtime function is always
/// markes as `#[inline]`.
///
/// See [`const_eval_select()`] for the rules and requirements around that intrinsic.
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
            if ($first) >= (static_max!(@inner $(,$rest)*))
            { ($first) }
            else
            { static_max!(@inner $(,$rest)*) }
         } };
    (@inner, $expr:expr) => { const { ($expr) } };
    (@inner, $first:expr $(,$rest:expr)*)
        => { const {
            if ($first) >= (static_max!(@inner $(,$rest)*))
            { ($first) }
            else
            { static_max!(@inner $(,$rest)*) }
         } };
}

#[macro_export]
macro_rules! max {
    () => { 0_usize };
    ($expr:expr) => { { ($expr) } };
    ($first:expr $(,$rest:expr)*)
        => { {
            if ($first) >= (max!(@inner $(,$rest)*))
            { ($first) }
            else
            { max!(@inner $(,$rest)*) }
         } };
    (@inner, $expr:expr) => { { ($expr) } };
    (@inner, $first:expr $(,$rest:expr)*)
        => { {
            if ($first) >= (max!(@inner $(,$rest)*))
            { ($first) }
            else
            { max!(@inner $(,$rest)*) }
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

#[cfg(test)]
mod tests
{
	use proc_macro_utils::array_size;

	use super::*;

	macro_rules! MY_ARRAY_CONSTANT {
		($callback:tt) => {
			callback!($callback([0, 1, 2, 3, 4, 5]))
		};
		() => {
			[0, 1, 2, 3, 4, 5]
		};
	}

	macro_rules! MY_MACRO {
        ($($tokens:tt)*) => {
            callback!(
                $($tokens)*(
                    1, 2, 3, 4, 5, 6
                )
            )
        };
    }

	macro_rules! MY_COOL_CALLBACK1 {
        ($($args:tt)*) => {
            static_max!($($args)*)
        };
    }

	macro_rules! MY_COOL_CALLBACK2 {
        ($($args:tt)*) => {
            (($($args)*) * 2)
        };
    }

	#[test]
	fn callback_test()
	{
		assert_eq!(MY_ARRAY_CONSTANT!(identity_expand), MY_ARRAY_CONSTANT!());
		assert_eq!(MY_ARRAY_CONSTANT!(array_size), MY_ARRAY_CONSTANT!().len());
		assert_eq!(MY_MACRO!(MY_COOL_CALLBACK1), 6);
		assert_eq!(MY_MACRO!(@foreach @delim(+) MY_COOL_CALLBACK2), 42);
	}

	#[test]
	fn identity_expand_test()
	{
		assert_eq!(
			identity_expand!("The quick brown fox blah blah blah"),
			"The quick brown fox blah blah blah"
		);
	}

	#[test]
	fn count_exprs_test()
	{
		assert_eq!(count_exprs!("1"), 1);
		assert_eq!(count_exprs!("1", "2"), 2);
		assert_eq!(count_exprs!("1", "2", "3"), 3);
		assert_eq!(count_exprs!("1", "2", "3", "4"), 4);
		assert_eq!(count_exprs!("1", "2", "3", "4", 5), 5);
	}

	#[test]
	fn static_max_test()
	{
		assert_eq!(static_max!(0, 8, 18, 2, 4235468, 1), 4235468);
		assert_eq!(static_max!(0, 8, 18, 2, 18, 1), 18);
	}
}
