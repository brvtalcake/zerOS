#![allow(non_snake_case)]
#![no_std]
#![no_main]
#![feature(decl_macro)]
#![feature(unboxed_closures, fn_traits)] // for crate 'overloadable' and overloadf
#![feature(const_slice_make_iter)]
#![feature(const_trait_impl)]
#![feature(generic_arg_infer)]
#![feature(sync_unsafe_cell)]
#![feature(trait_alias)]
#![feature(variant_count)]

#[macro_use]
extern crate macro_utils;

#[macro_use]
extern crate proc_macro_utils;

pub mod arch;
pub mod error;
pub mod init;
pub mod kernel;
#[macro_use]
pub mod logging;
pub mod panic;
pub mod utils;

use crate::arch::target::cpu::misc::hcf;

fn kmain() -> !
{
	if let Some(framebuffer_response) =
		init::bootloaders::limine::FRAMEBUFFER_REQUEST.get_response()
	{
		if let Some(framebuffer) = framebuffer_response.framebuffers().next()
		{
			for i in 0..100_u64
			{
				// Calculate the pixel offset using the framebuffer information we obtained
				// above. We skip `i` scanlines (pitch is provided in bytes) and add `i * 4`
				// to skip `i` pixels forward.
				let pixel_offset = i * framebuffer.pitch() + i * 4;

				// Write 0xFFFFFFFF to the provided pixel offset to fill it white.
				unsafe {
					framebuffer
						.addr()
						.add(pixel_offset as usize)
						.cast::<u32>()
						.write(0xffffffff)
				};
			}
		}
	}

	hcf()
}
