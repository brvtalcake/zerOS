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
}