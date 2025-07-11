mod public;
pub mod traits;

pub use public::*;

pub macro tuint($($tokens:tt)*)
{
    ::typenum::U<{ $crate::meta::val!($($tokens)*) }>
}

pub macro val($($tokens:tt)*)
{
    <$($tokens)*>::VALUE
}