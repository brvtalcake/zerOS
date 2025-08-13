use core::{
	error::Error as CoreError,
	marker::{PhantomData, Tuple}
};

#[const_trait]
pub trait Construct<Args>: Sized
where
	Args: Tuple
{
	extern "rust-call" fn construct(args: Args) -> Self;
}

#[const_trait]
pub trait TryConstruct<Args>: Sized
where
	Args: Tuple
{
	type Error: CoreError + Sized;

	extern "rust-call" fn try_construct(args: Args) -> Result<Self, Self::Error>;
}

pub struct Constructor<T>(PhantomData<T>);

impl<T, Args: Tuple> const FnOnce<Args> for Constructor<T>
where
	T: [const] Construct<Args>
{
	type Output = T;

	extern "rust-call" fn call_once(self, args: Args) -> Self::Output
	{
		T::construct(args)
	}
}

impl<T, Args: Tuple> const FnMut<Args> for Constructor<T>
where
	T: [const] Construct<Args>
{
	extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output
	{
		T::construct(args)
	}
}

impl<T, Args: Tuple> const Fn<Args> for Constructor<T>
where
	T: [const] Construct<Args>
{
	extern "rust-call" fn call(&self, args: Args) -> Self::Output
	{
		T::construct(args)
	}
}

pub struct FallibleConstructor<T>(PhantomData<T>);

impl<T, Args: Tuple> const FnOnce<Args> for FallibleConstructor<T>
where
	T: [const] TryConstruct<Args>
{
	type Output = Result<T, <T as TryConstruct<Args>>::Error>;

	extern "rust-call" fn call_once(self, args: Args) -> Self::Output
	{
		T::try_construct(args)
	}
}

impl<T, Args: Tuple> const FnMut<Args> for FallibleConstructor<T>
where
	T: [const] TryConstruct<Args>
{
	extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output
	{
		T::try_construct(args)
	}
}

impl<T, Args: Tuple> const Fn<Args> for FallibleConstructor<T>
where
	T: [const] TryConstruct<Args>
{
	extern "rust-call" fn call(&self, args: Args) -> Self::Output
	{
		T::try_construct(args)
	}
}

pub trait Constructible: Sized
{
	#[allow(non_upper_case_globals)]
	const new: Constructor<Self> = Constructor(PhantomData);
}

pub trait FailliblyConstructible: Sized
{
	#[allow(non_upper_case_globals)]
	const try_new: FallibleConstructor<Self> = FallibleConstructor(PhantomData);
}
