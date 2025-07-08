#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(trait_alias)]
#![feature(decl_macro)]

use core::convert::Infallible;

extern crate alloc;

mod text;
pub use text::*;

mod port;
pub use port::*;

// TODO: in-memory contiguous IO variants
