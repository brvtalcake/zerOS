#![feature(concat_idents)]
#![no_std]

#![recursion_limit = "1024"]

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

#[cfg(test)]
mod tests
{
    use super::*;
    use proc_macro_utils::array_size;

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
