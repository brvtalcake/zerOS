#![allow(non_snake_case)]
#![no_std]
#![no_main]
#![feature(min_specialization)]
#![feature(link_llvm_intrinsics)]
#![feature(decl_macro)]
#![feature(unboxed_closures, fn_traits)] // for crate 'overloadable' and overloadf
#![feature(const_slice_make_iter)]
#![feature(const_trait_impl)]
#![feature(generic_arg_infer)]
#![feature(sync_unsafe_cell)]
#![feature(trait_alias)]
#![feature(variant_count)]
#![feature(transmutability)]
#![feature(allocator_api)]
#![feature(unsafe_cell_access)]
#![feature(nonzero_ops)]
#![feature(trivial_bounds)]
#![feature(exclusive_wrapper)]
#![feature(ptr_as_ref_unchecked)]
#![feature(phantom_variance_markers)]
#![feature(slice_ptr_get)]
#![feature(likely_unlikely)]
#![feature(used_with_arg)]
#![feature(generic_const_exprs)]
#![feature(portable_simd)]
#![recursion_limit = "512"]
// TODO: change the compile flags to use vector extensions IN-KERNEL

extern crate alloc;

#[macro_use]
extern crate impls;

#[macro_use]
extern crate macro_utils;

#[macro_use]
extern crate proc_macro_utils;

pub mod arch;
pub mod init;
pub mod kernel;
pub mod llvm;
pub mod panic;
pub mod unwinding;
pub mod utils;

use crate::{arch::target::cpu::misc::hcf, kernel::linker::LinkerSym};

#[allow(dead_code)]
static UNIFONT: &[u8] = include_bytes!("../assets/font/unifont-16.0.04.otf");
#[allow(dead_code)]
static LOGO: &[u8] =
	include_bytes!("../assets/logo/zeros-high-resolution-logo-white-transparent.svg");

fn kmain() -> !
{
	// TODO: add some kind of "Framebuffer" trait
	// TODO: implement something like a virtual terminal structure
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
