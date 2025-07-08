use core::convert::Infallible;

use sealed::sealed;
use zerOS_macro_utils::min;

/// A trait implemented by devices which we can read from by means of reading
/// some memory-mapped registers or (only for x86) port IO (e.g. the `in{b|...}`
/// instruction)
pub trait KernelPortInput<T: FromPortIO>
{
	type Error = Infallible;

	/// Read a value from a port.
	fn read(&self) -> Result<T, Self::Error>;

	/// Read multiple successive values from a port.
	///
	/// # Return value
	/// The default implementation just reads the values one-by-one and
	/// forwards the potential errors.
	fn read_multiple(&self, slice: &mut [T], max: Option<usize>) -> Result<(), Self::Error>
	{
		let slice_len = slice.len();
		let max = max.map_or(slice_len, |requested| min!(requested, slice_len));
		for i in 0..max
		{
			unsafe {
				*slice.get_unchecked_mut(i) = self.read()?;
			}
		}
		Ok(())
	}

	/// Read from the port while a predicate holds.
	///
	/// # Warning
	/// The last value read (i.e. when the predicate becomes false), can not be
	/// re-read afterwards.
	///
	/// # Return value
	/// A `Result` with the amount of values that could be read successfully, or
	/// an error if a read error occurred.
	fn read_while(
		&self,
		slice: &mut [T],
		mut predicate: &dyn FnMut(T) -> bool
	) -> Result<usize, <Self as KernelPortInput<T>>::Error>
	{
		let (mut i, max) = (0, slice.len());
		while i < max
		{
			let val = self.read()?;
			if !predicate(val)
			{
				return Ok(i);
			}
			// SAFETY: the loop invariant holds here
			unsafe {
				*slice.get_unchecked_mut(i) = val;
			}
			i += 1;
		}
		Ok(i)
	}

	/// Read from the port while a predicate does _*not*_ hold.
	///
	/// # Warning
	/// The last value read (i.e. when the predicate becomes true), can not be
	/// re-read afterwards.
	///
	/// # Return value
	/// A `Result` with the amount of values that could be read successfully, or
	/// an error if a read error occurred.
	fn read_until(
		&self,
		slice: &mut [T],
		mut predicate: &dyn FnMut(T) -> bool
	) -> Result<usize, <Self as KernelPortInput<T>>::Error>
	{
		self.read_while(slice, |val| !predicate(val))
	}
}

/// A trait implemented by devices which we can write to by means of writing to
/// some memory-mapped registers or (only for x86) port IO (e.g. the
/// `out{b|...}` instruction)
pub trait KernelPortOutput<T: ToPortIO>
{
	type Error = Infallible;

	/// Write a value to the port.
	fn write(&self, value: T) -> Result<(), Self::Error>;

	/// Write multiple value to the port.
	///
	/// # Return value
	/// The default implementation just writes the values one-by-one and
	/// forwards the potential errors.
	fn write_multiple(&self, slice: &[T], max: Option<usize>) -> Result<(), Self::Error>
	{
		let slice_len = slice.len();
		let max = max.map_or(slice_len, |requested| min!(requested, slice_len));
		for i in 0..max
		{
			// SAFETY: unsafe here is fine since we just verified that the index is
			// in-bounds
			unsafe {
				self.write(*slice.get_unchecked(i))?;
			}
		}
		Ok(())
	}
}

pub trait KernelPortIO<T: FromPortIO + ToPortIO> = KernelPortInput<T> + KernelPortOutput<T>;

pub trait KernelPortInputExt<T: FromPortIO>: KernelPortInput<T> {}

pub trait KernelPortOutputExt<T: ToPortIO>: KernelPortOutput<T> {}

pub trait KernelPortIOExt<T: FromPortIO + ToPortIO> =
	KernelPortInputExt<T> + KernelPortOutputExt<T>;

macro impl_sealed_for(impl $tr:path for [$($types:ty),* $(,)?])
{
    $(
        #[sealed]
        impl $tr for $types
        {}
    )*
}

#[sealed]
pub trait FromPortIO {}

// TODO: the following implementations must be implemented only if target
// actually can support it
impl_sealed_for! {
	impl FromPortIO for [u8, i8, u16, i16, u32, i32, u64, i64]
}

#[sealed]
pub trait ToPortIO {}

// TODO: the following implementations must be implemented only if target
// actually can support it
impl_sealed_for! {
	impl ToPortIO for [u8, i8, u16, i16, u32, i32, u64, i64]
}
