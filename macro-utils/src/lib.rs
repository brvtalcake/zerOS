#![feature(concat_idents)]

#![no_std]

#[macro_export]
macro_rules! callback {
    ($callback:ident( $($args:tt)* )) => {
        $callback!( $($args)* )
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

#[cfg(test)]
mod tests
{
    use super::*;
    use proc_macro_utils::array_size;

    macro_rules! MY_ARRAY_CONSTANT {
        ($callback:tt) => {
            callback!($callback([0, 1, 2, 3, 4, 5]))
        };
        () => { [0, 1, 2, 3, 4, 5] };
    }

    #[test]
    fn callback_test()
    {
        assert_eq!(MY_ARRAY_CONSTANT!(identity_expand), MY_ARRAY_CONSTANT!());
        assert_eq!(MY_ARRAY_CONSTANT!(array_size), MY_ARRAY_CONSTANT!().len());
    }

    #[test]
    fn identity_expand_test()
    {
        assert_eq!(identity_expand!("The quick brown fox blah blah blah"), "The quick brown fox blah blah blah");
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
}