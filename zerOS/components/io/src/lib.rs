#![recursion_limit = "256"]
#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(trait_alias)]
#![feature(pointer_is_aligned_to)]
#![feature(associated_type_defaults)]
#![feature(decl_macro)]
#![feature(bstr)]
#![feature(unboxed_closures)]
#![feature(allocator_api)]
#![feature(coerce_unsized)]
#![feature(fn_traits)]

use alloc::{format, string::ToString};
use core::{
	any,
	fmt::{Debug, Display},
	panic::Location as SourceLocation
};

use downcast_rs::{Downcast, impl_downcast};
use impls::impls;
use overloadf::overload;
use thiserror::Error;
use zerOS_static_assertions::static_assert;
use zerOS_utils::VoidResult;

extern crate alloc;

mod block;
mod memory;
mod port;
mod text;
mod wrapper;

const IO_ERROR_DISPLAY_PREFIX: &'static str = "io error:";

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StreamKind
{
	/// A stream that you can read from and write to unit-by-unit (e.g. byte by
	/// byte), whether it is through io-port assembly instructions (as for x86),
	/// or with memory-mapped registers
	Port,
	/// An in-memory contiguous stream, akin to an mmaped-file (but could
	/// reference any valid memory)
	// TODO: add a `VirtAddr` member
	Memory,
	/// Block io, e.g. disk io
	Block
}
static_assert!(impls!(StreamKind: Display & Debug & ToString));

impl StreamKind
{
	const fn prefix(&self) -> &'static str
	{
		match self
		{
			Self::Memory => "an",
			_ => "a"
		}
	}

	const fn describe(&self) -> &'static str
	{
		match self
		{
			Self::Port => "port io",
			Self::Memory => "in-memory (contiguous) io",
			Self::Block => "block io"
		}
	}
}

impl Display for StreamKind
{
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
	{
		f.write_str(format!("{} `{}`", self.prefix(), self.describe()).as_str())
	}
}

#[derive(Debug, Error)]
pub enum IOError
{
	#[error(
		"{IO_ERROR_DISPLAY_PREFIX} reading only {provided} elements of type `{elem_type}` from \
		 this source would loose bytes (at least {minimum} bytes should be read)"
	)]
	BufferTooSmall
	{
		provided:  usize,
		elem_type: &'static str,
		minimum:   usize
	},
	#[error(
		"{IO_ERROR_DISPLAY_PREFIX} unhandled type `{type_name}` in function `{fn_name}` at \
		 {location}"
	)]
	UnhandledType
	{
		type_name: &'static str,
		fn_name:   &'static str,
		location:  &'static SourceLocation<'static>
	},
	#[error(
		"{IO_ERROR_DISPLAY_PREFIX} method `{method_name}` (at {location}) can be called with \
		 {expected} kind of underlying stream, but the current one is {actual}"
	)]
	WrongStreamKind
	{
		method_name: &'static str,
		expected:    StreamKind,
		actual:      StreamKind,
		location:    &'static SourceLocation<'static>
	},
	#[error("{IO_ERROR_DISPLAY_PREFIX} {0}")]
	Other(anyhow::Error)
}

#[overload]
impl IOError
{
	pub const fn buffer_too_small<T>(buffer: &T, minimum: usize) -> Self
	{
		Self::BufferTooSmall {
			provided: 1,
			elem_type: any::type_name_of_val(buffer),
			minimum
		}
	}

	pub const fn buffer_too_small<T>(buffer: &[T], minimum: usize) -> Self
	{
		Self::BufferTooSmall {
			provided: buffer.len(),
			elem_type: any::type_name::<T>(),
			minimum
		}
	}

	pub fn other(other: impl Into<anyhow::Error>) -> Self
	{
		Self::Other(other.into())
	}

	#[track_caller]
	pub fn unhandled_type<T>(value: T, fn_name: &'static str) -> Self
	{
		Self::UnhandledType {
			type_name: any::type_name_of_val(&value),
			fn_name,
			location: SourceLocation::caller()
		}
	}

	#[track_caller]
	pub fn wrong_stream_kind(
		method_name: &'static str,
		expected: StreamKind,
		actual: StreamKind
	) -> Self
	{
		Self::WrongStreamKind {
			method_name,
			expected,
			actual,
			location: SourceLocation::caller()
		}
	}
}

pub trait KernelInput: Downcast
{
	/// Fills the buffer with read bytes
	fn read_bytes(&mut self, buffer: &mut [u8]) -> VoidResult<IOError>;
}
impl_downcast!(KernelInput);

pub trait KernelOutput: Downcast
{
	/// Writes bytes
	fn write_bytes(&mut self, bytes: &[u8]) -> VoidResult<IOError>;
}
impl_downcast!(KernelOutput);

pub trait KernelIO: KernelInput + KernelOutput {}

impl<T: KernelInput + KernelOutput> KernelIO for T {}

// mod impls;
//
// verify_impls! {
// 	from + to: u8 i8 u16 i16 u32 i32 u64 i64
//}

// TODO: block IO variants
pub use block::*;
// TODO: in-memory contiguous IO variants
pub use memory::*;
pub use port::*;
pub use text::*;
pub use wrapper::*;
