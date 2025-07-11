use eager2::{eager, eager_macro};
use sealed::sealed;
use typenum::{False, True};

#[eager_macro]
macro_rules! impl_sealed_for
{
    (
        impl $tr:path for [$ty:ty]
            with $callback:ident($($args:tt)*)
    ) => {
        #[sealed]
        impl $tr for $ty
        {
            $callback!($ty, ($($args)*))
        }
    };

    (
        impl $tr:path for [$first:ty, $($others:ty),* $(,)?]
            with $callback:ident($($args:tt)*)
    ) => {
        impl_sealed_for! {
            impl $tr for [$first]
                with $callback($($args)*)
        }
        impl_sealed_for! {
            impl $tr for [$($others),* $(,)?]
                with $callback($($args)*)
        }
    };
}

#[eager_macro]
macro_rules! impl_from_io {
	($ty:ty, ($fundamental:ty)) => {
		type IsFundamental = $fundamental;
	};
}

#[eager_macro]
macro_rules! impl_to_io {
	($ty:ty, ($fundamental:ty)) => {
		type IsFundamental = $fundamental;
	};
}

eager! {
	// TODO: the following implementations must be implemented only if target
	// actually can support it
	impl_sealed_for! {
		impl super::FromIO for [u8, i8, u16, i16, u32, i32, u64, i64]
			with impl_from_io(True)
	}

	// TODO: the following implementations must be implemented only if target
	// actually can support it
	impl_sealed_for! {
		impl super::ToIO for [u8, i8, u16, i16, u32, i32, u64, i64]
			with impl_to_io(True)
	}
}

pub(crate) macro verify_impls
{
    (from: $($types:ty)*) => {
        $(
            ::zerOS_static_assertions::static_assert!(
                ::impls::impls!(
                    $types: $crate::FromIO
                )
            );
        )*
    },

    (to: $($types:ty)*) => {
        $(
            ::zerOS_static_assertions::static_assert!(
                ::impls::impls!(
                    $types: $crate::ToIO
                )
            );
        )*
    },

    (from + to: $($types:ty)*) => {
        $(
            ::zerOS_static_assertions::static_assert!(
                ::impls::impls!(
                    $types: ($crate::FromIO) & ($crate::ToIO)
                )
            );
        )*
    },

    ($($_:tt)*) => { ::zerOS_static_assertions::static_assert!(false); },
}
