#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(trait_alias)]
#![feature(pointer_is_aligned_to)]
#![feature(associated_type_defaults)]
#![feature(decl_macro)]
#![feature(bstr)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]

use core::{any, cell::LazyCell, error::Error as CoreError, panic::Location as SourceLocation};

use downcast_rs::Downcast;
use overloadf::overload;
use thiserror::Error;
use zerOS_utils::{VoidResult, function};

extern crate alloc;

mod block;
mod memory;
mod port;
mod text;

//#[sealed]
// pub trait FundamentalIO: 'static {}

const IO_ERROR_DISPLAY_PREFIX: &'static str = "io error:";

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
		"{IO_ERROR_DISPLAY_PREFIX} unhandled type `{type_name}` in function `{fn_name}` at {location}"
	)]
	UnhandledType
	{
		type_name: &'static str,
		fn_name:   &'static str,
		location:  &'static SourceLocation<'static>
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
}

pub trait KernelInputBase: Downcast
{
	/// Fills the buffer with read bytes
	fn read_bytes(&mut self, buffer: &mut [u8]) -> VoidResult<IOError>;
}

pub trait KernelOutputBase: Downcast
{
	/// Writes bytes
	fn write_bytes(&mut self, bytes: &[u8]) -> VoidResult<IOError>;
}

pub trait KernelIOBase = KernelInputBase + KernelOutputBase;

pub trait KernelInput: KernelInputBase {}

pub trait KernelOutput: KernelOutputBase {}

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
