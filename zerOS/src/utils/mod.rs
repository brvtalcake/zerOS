use core::mem::{self, Assume, TransmuteFrom};

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
