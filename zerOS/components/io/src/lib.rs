#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(trait_alias)]
#![feature(associated_type_defaults)]
#![feature(decl_macro)]

use core::{
	convert::Infallible,
	error::Error as CoreError,
	ops::{Deref, DerefMut}
};

use downcast_rs::Downcast;
use sealed::sealed;
use typenum::{Bit, True, TypeArray};
use zerocopy::{FromBytes, IntoBytes};

use crate::impls::verify_impls;

extern crate alloc;

#[sealed]
pub trait FromIO: Default + FromBytes + IntoBytes + 'static
{
	type IsFundamental: Bit;
}

#[sealed]
pub trait ToIO: Default + FromBytes + IntoBytes + 'static
{
	type IsFundamental: Bit;
}

pub trait FundamentalFromIO = FromIO<IsFundamental = True>;
pub trait FundamentalToIO = ToIO<IsFundamental = True>;

// TODO: serializers should be implemented as streaming wrappers around KernelIO
// implementers

pub trait KernelInputBase: Downcast
{
	type OptimalInput: FundamentalFromIO;
	type MinimalInput: FundamentalFromIO;
	type InputError: CoreError;

	fn read_optimal(&mut self, buffer: &mut Self::OptimalInput) -> Result<(), Self::InputError>;

	fn read_minimal(&mut self, buffer: &mut Self::MinimalInput) -> Result<(), Self::InputError>;
}

pub trait KernelOutputBase: Downcast
{
	type OptimalOutput: FundamentalToIO;
	type MinimalOutput: FundamentalToIO;
	type OutputError: CoreError;

	fn write_optimal(&mut self, value: &Self::OptimalOutput) -> Result<(), Self::OutputError>;

	fn write_minimal(&mut self, value: &Self::MinimalOutput) -> Result<(), Self::OutputError>;
}

pub trait KernelIOBase = KernelInputBase + KernelOutputBase;

mod impls;

verify_impls! {
	from + to: u8 i8 u16 i16 u32 i32 u64 i64
}

mod text;
pub use text::*;

mod port;
pub use port::*;

// TODO: in-memory contiguous IO variants
mod memory;
pub use memory::*;
// TODO: block IO variants
mod block;
pub use block::*;
