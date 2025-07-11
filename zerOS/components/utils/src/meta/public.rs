use core::{marker::PhantomInvariant, mem};

use typenum::{Const, ToUInt};

/// Trait to retrieve the size of a type.
///
/// # Examples
/// TODO
pub struct SizeOf<T: ?Sized>(PhantomInvariant<T>);

impl<T: Sized> SizeOf<T>
where
	Const<{ mem::size_of::<T>() }>: ToUInt
{
	// type Value = U<{ mem::size_of::<T>() }>;

	pub const VALUE: usize = mem::size_of::<T>();
}
