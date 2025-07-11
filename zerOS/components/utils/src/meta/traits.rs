#[cfg(false)]
mod imp
{
	use sealed::sealed;
	use typenum::{Const, ToUInt, U};

	#[sealed]
	trait SizeOfHelper
	{
		const VALUE: usize = 0;
	}

	#[sealed]
	impl<T: ?Sized> SizeOfHelper for super::super::SizeOf<T> {}

	#[sealed]
	pub trait SizeOfTrait: SizeOfHelper
	where
		Const<{ <Self as SizeOfHelper>::VALUE }>: ToUInt
	{
		const VALUE: usize = <Self as SizeOfHelper>::VALUE;
		type Value = U<{ <Self as SizeOfHelper>::VALUE }>;
	}

	#[sealed]
	impl<T: ?Sized> SizeOfTrait for super::super::SizeOf<T> where
		Const<{ <Self as SizeOfHelper>::VALUE }>: ToUInt
	{
	}
}

#[cfg(true)]
mod imp
{
	use sealed::sealed;

	#[sealed]
	trait SizeOfTrait
	{
		const VALUE: usize = 0;
	}

	#[sealed]
	impl<T: ?Sized> SizeOfTrait for super::super::SizeOf<T> {}
}
