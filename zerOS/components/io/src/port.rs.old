use core::{convert::Infallible, error::Error as CoreError, mem};

use eager2::*;
use sealed::sealed;
use typenum::{Bit, False, GrEq, True, U, op};
use zerOS_macro_utils::{min, static_max, static_min};
use zerOS_static_assertions::static_assert;
use zerocopy::{transmute_mut, transmute_ref};

use crate::{FromIO, KernelInputBase, KernelOutputBase, ToIO};

/// A trait implemented by devices which we can read from by means of reading
/// some memory-mapped registers or (only for x86) port IO (e.g. the `in{b|...}`
/// instruction)
pub trait KernelPortInput<T: FromIO = Self::OptimalInput>: KernelInputBase
{
	/// Read a value from a port.
	fn read_port(&mut self, buffer: &mut T) -> Result<(), Self::InputError>
	{
		const TSIZE: usize = mem::size_of::<T>();
		const OPTSIZE: usize = mem::size_of::<Self::OptimalInput>();

		let buffer_bytes: &mut [u8] = transmute_mut!(buffer);
		if TSIZE < OPTSIZE
		{
			const MINSIZE: usize = mem::size_of::<Self::MinimalInput>();
			static_assert!(TSIZE >= MINSIZE);
			static_assert!(TSIZE % MINSIZE == 0);
			const NEEDED: usize = TSIZE / MINSIZE;

			let mut minimal = Default::default();
			for i in 0..NEEDED
			{
				self.read_minimal(&mut minimal)?;
				unsafe {
					let addr: *mut Self::MinimalInput =
						(&raw mut *buffer_bytes.get_unchecked_mut(i * MINSIZE)).cast();
					addr.write_unaligned(minimal);
				}
			}
		}
		else
		{
			debug_assert!(TSIZE >= OPTSIZE);
			debug_assert_eq!(TSIZE % OPTSIZE, 0);
			const NEEDED: usize = TSIZE / OPTSIZE;

			let mut opt = Default::default();
			for i in 0..NEEDED
			{
				self.read_optimal(&mut opt)?;
				unsafe {
					let addr: *mut Self::OptimalInput =
						(&raw mut *buffer_bytes.get_unchecked_mut(i * OPTSIZE)).cast();
					addr.write_unaligned(opt);
				}
			}
		}

		Ok(())
	}

	/// Read multiple successive values from a port.
	///
	/// # Return value
	/// The default implementation just reads the values one-by-one and
	/// forwards the potential errors.
	fn read_port_multiple(
		&mut self,
		slice: &mut [T],
		max: Option<usize>
	) -> Result<usize, Self::InputError>
	{
		let slice_len = slice.len();
		let max = max.map_or(slice_len, |requested| min!(requested, slice_len));
		for i in 0..max
		{
			unsafe {
				self.read_port(slice.get_unchecked_mut(i))?;
			}
		}
		Ok(max)
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
	fn read_port_while(
		&mut self,
		slice: &mut [T],
		mut predicate: &dyn FnMut(&T) -> bool
	) -> Result<usize, Self::InputError>
	{
		let (mut i, max) = (0, slice.len());
		while i < max
		{
			let mut val = Default::default();
			self.read_port(&mut val)?;
			if !predicate(&val)
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
	fn read_port_until(
		&mut self,
		slice: &mut [T],
		mut predicate: &dyn FnMut(&T) -> bool
	) -> Result<usize, Self::InputError>
	{
		self.read_port_while(slice, |val| !predicate(val))
	}
}

/// A trait implemented by devices which we can write to by means of writing to
/// some memory-mapped registers or (only for x86) port IO (e.g. the
/// `out{b|...}` instruction)
pub trait KernelPortOutput<T: ToIO = Self::OptimalOutput>: KernelOutputBase
{
	/// Write a value to the port.
	fn write_port(&mut self, value: &T) -> Result<(), Self::OutputError>
	{
		const TSIZE: usize = mem::size_of::<T>();
		const OPTSIZE: usize = mem::size_of::<Self::OptimalOutput>();

		if TSIZE < OPTSIZE
		{
			const MINSIZE: usize = mem::size_of::<Self::MinimalOutput>();
			static_assert!(TSIZE >= MINSIZE);
			static_assert!(TSIZE % MINSIZE == 0);

			let value_bytes: &[Self::MinimalOutput] = transmute_ref!(value);
			for b in value_bytes
			{
				self.write_minimal(b)?;
			}
		}
		else
		{
			debug_assert!(TSIZE >= OPTSIZE);
			debug_assert_eq!(TSIZE % OPTSIZE, 0);

			let value_bytes: &[Self::OptimalOutput] = transmute_ref!(value);
			for b in value_bytes
			{
				self.write_optimal(b)?;
			}
		}

		Ok(())
	}

	/// Write multiple value to the port.
	///
	/// # Return value
	/// The default implementation just writes the values one-by-one and
	/// forwards the potential errors.
	fn write_port_multiple(
		&mut self,
		slice: &[T],
		max: Option<usize>
	) -> Result<usize, Self::OutputError>
	{
		let slice_len = slice.len();
		let max = max.map_or(slice_len, |requested| min!(requested, slice_len));
		for i in 0..max
		{
			// SAFETY: unsafe here is fine since we just verified that the index is
			// in-bounds
			unsafe {
				self.write_port(*slice.get_unchecked(i))?;
			}
		}
		Ok(max)
	}
}

pub trait KernelPortIO<T: FromIO + ToIO> = KernelPortInput<T> + KernelPortOutput<T>;

pub trait KernelPortInputExt<T: FromIO = Self::OptimalInput>: KernelPortInput<T>
{
	fn read_port_while_inclusive(
		&mut self,
		slice: &mut [T],
		mut predicate: &dyn FnMut(&T) -> bool
	) -> Result<usize, Self::InputError>
	{
		let (mut i, max) = (0, slice.len());
		while i < max
		{
			let mut val = Default::default();
			self.read_port(&mut val)?;
			if !predicate(&val)
			{
				unsafe {
					*slice.get_unchecked_mut(i) = val;
				}
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

	fn read_port_until_inclusive(
		&mut self,
		slice: &mut [T],
		mut predicate: &dyn FnMut(&T) -> bool
	) -> Result<usize, Self::InputError>
	{
		self.read_port_while_inclusive(slice, |val| !predicate(val))
	}
}

pub trait KernelPortOutputExt<T: ToIO = Self::OptimalOutput>: KernelPortOutput<T> {}

pub trait KernelPortIOExt<T: FromIO + ToIO> = KernelPortInputExt<T> + KernelPortOutputExt<T>;

pub trait AsKernelPortInput<T: FromIO>: KernelInputBase
{
	fn as_kernel_port_input(&self) -> Option<&dyn KernelPortInput<T>>;

	fn as_mut_kernel_port_input(&mut self) -> Option<&mut dyn KernelPortInput<T>>;
}

pub trait AsKernelPortOutput<T: FromIO>: KernelOutputBase
{
	fn as_kernel_port_output(&self) -> Option<&dyn KernelPortOutput<T>>;

	fn as_mut_kernel_port_output(&mut self) -> Option<&mut dyn KernelPortOutput<T>>;
}
