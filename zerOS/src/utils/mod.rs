use core::mem::{self, Assume, TransmuteFrom};

pub use rustc_demangle::{demangle, try_demangle};

pub mod str;

pub macro likely($boolean:expr) {
	::core::hint::likely(($boolean) as bool)
}

pub macro unlikely($boolean:expr) {
	::core::hint::unlikely(($boolean) as bool)
}

#[macro_export]
macro_rules! alignment_of {
	(    ) => {
		0
	};
	(u8  ) => {
		1
	};
	(i8  ) => {
		1
	};
	(f8  ) => {
		1
	};
	(u16 ) => {
		2
	};
	(i16 ) => {
		2
	};
	(f16 ) => {
		2
	};
	(u32 ) => {
		4
	};
	(i32 ) => {
		4
	};
	(f32 ) => {
		4
	};
	(u64 ) => {
		8
	};
	(i64 ) => {
		8
	};
	(f64 ) => {
		8
	};
	(u128) => {
		16
	};
	(i128) => {
		16
	};
	(f128) => {
		16
	};
	($other:ty) => {
		const { align_of::<$other>() }
	};
}

#[macro_export]
/// # TODO
/// This is technically `* __CHAR_BIT__` but no idea how to get that info from
/// rust
macro_rules! width_of {
	($typ:ty) => {
		($crate::size_of!($typ) * 8)
	};
}

#[macro_export]
macro_rules! size_of {
	(    ) => {
		0
	};
	(u8  ) => {
		1
	};
	(i8  ) => {
		1
	};
	(f8  ) => {
		1
	};
	(u16 ) => {
		2
	};
	(i16 ) => {
		2
	};
	(f16 ) => {
		2
	};
	(u32 ) => {
		4
	};
	(i32 ) => {
		4
	};
	(f32 ) => {
		4
	};
	(u64 ) => {
		8
	};
	(i64 ) => {
		8
	};
	(f64 ) => {
		8
	};
	(u128) => {
		16
	};
	(i128) => {
		16
	};
	(f128) => {
		16
	};
	($other:ty) => {
		const { size_of::<$other>() }
	};
}

#[macro_export]
macro_rules! ensure_is_fragment_kind {
	(expr, ($($expansion:tt)*), ($good:expr)) => {
		$($expansion)*
	};
	(expr, ($($expansion:tt)*), ($($bad:tt)*)) => {
		compile_error!(
			concat!(
				"\"", stringify!($($bad)*), "\"",
				" is not a valid macro expression (\":expr\")"
			)
		)
	};
	(expr, ($good:expr)) => {
		$good
	};
	(expr, ($($bad:tt)*)) => {
		compile_error!(
			concat!(
				"\"", stringify!($($bad)*), "\"",
				" is not a valid macro expression (\":expr\")"
			)
		)
	};

	(type, ($($expansion:tt)*), ($good:ty)) => {
		$($expansion)*
	};
	(type, ($($expansion:tt)*), ($($bad:tt)*)) => {
		compile_error!(
			concat!(
				"\"", stringify!($($bad)*), "\"",
				" is not a valid macro type (\":ty\")"
			)
		)
	};
	(type, ($good:ty)) => {
		$good
	};
	(type, ($($bad:tt)*)) => {
		compile_error!(
			concat!(
				"\"", stringify!($($bad)*), "\"",
				" is not a valid macro type (\":ty\")"
			)
		)
	};
}

#[macro_export]
macro_rules! func_at {
	(@GET_EXPR[$($address:tt)*]) => {
		core::mem::transmute::<*const (), _>(
			$crate::ensure_is_fragment_kind!(expr, ($($address)*)) as _
		)
	};

	(@GET_EXPR[$($address:tt)*] as $($fnty:tt)*) => {
		core::mem::transmute::<
			*const (),
			$crate::ensure_is_fragment_kind!(type, ($($fnty)*))
		>(
			$crate::ensure_is_fragment_kind!(expr, ($($address)*)) as _
		)
	};

	(@GET_EXPR[$($tokens:tt)*] $next:tt $($rest:tt)*) => {
		$crate::func_at!(@GET_EXPR[$($tokens)* $next] $($rest)*)
	};

	($($tokens:tt)+) => {
		$crate::func_at!(@GET_EXPR[] $($tokens)+)
	};
}

pub fn assume_aligned<T: Copy>(src: T) -> T
where
	T: TransmuteFrom<T, { Assume::ALIGNMENT }>
{
	unsafe { <T as TransmuteFrom<T, { Assume::ALIGNMENT }>>::transmute(src) }
}

pub const unsafe fn with_lifetime<'from, 'to, T: ?Sized>(from: &'from T) -> &'to T
{
	unsafe { mem::transmute(from) }
}

pub const unsafe fn with_lifetime_mut<'from, 'to, T: ?Sized>(from: &'from mut T) -> &'to mut T
{
	unsafe { mem::transmute(from) }
}
