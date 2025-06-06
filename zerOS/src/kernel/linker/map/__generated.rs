// --- SECTIONINFO AUTOGENERATED BY
// /home/axel/Documents/programmation/osdev/zerOS/scripts/gensectioninfo.py,
// START ---
pub mod __linker_symbols
{
	#![allow(dead_code)]
	#![allow(non_upper_case_globals)]
	use lazy_static::lazy_static;

	use super::super::LinkerSym;
	pub const zerOS_section_count: usize = 8;

	unsafe extern "C" {
		pub unsafe static zerOS_kernel_start: LinkerSym;
		pub unsafe static zerOS_kernel_end: LinkerSym;
	}

	unsafe extern "C" {
		pub unsafe static zerOS_text_start: LinkerSym;
		pub unsafe static zerOS_text_end: LinkerSym;
	}
	lazy_static! {
		#[allow(non_upper_case_globals)]
		pub static ref zerOS_text_size: usize = {
			unsafe {
				(zerOS_text_end   as usize) -
				(zerOS_text_start as usize)
			}
		};
	}

	unsafe extern "C" {
		pub unsafe static zerOS_bootcode_start: LinkerSym;
		pub unsafe static zerOS_bootcode_end: LinkerSym;
	}
	lazy_static! {
		#[allow(non_upper_case_globals)]
		pub static ref zerOS_bootcode_size: usize = {
			unsafe {
				(zerOS_bootcode_end   as usize) -
				(zerOS_bootcode_start as usize)
			}
		};
	}

	unsafe extern "C" {
		pub unsafe static zerOS_ctors_init_array_start: LinkerSym;
		pub unsafe static zerOS_ctors_init_array_end: LinkerSym;
	}
	lazy_static! {
		#[allow(non_upper_case_globals)]
		pub static ref zerOS_ctors_init_array_size: usize = {
			unsafe {
				(zerOS_ctors_init_array_end   as usize) -
				(zerOS_ctors_init_array_start as usize)
			}
		};
	}

	unsafe extern "C" {
		pub unsafe static zerOS_rodata_start: LinkerSym;
		pub unsafe static zerOS_rodata_end: LinkerSym;
	}
	lazy_static! {
		#[allow(non_upper_case_globals)]
		pub static ref zerOS_rodata_size: usize = {
			unsafe {
				(zerOS_rodata_end   as usize) -
				(zerOS_rodata_start as usize)
			}
		};
	}

	unsafe extern "C" {
		pub unsafe static zerOS_eh_frame_hdr_start: LinkerSym;
		pub unsafe static zerOS_eh_frame_hdr_end: LinkerSym;
	}
	lazy_static! {
		#[allow(non_upper_case_globals)]
		pub static ref zerOS_eh_frame_hdr_size: usize = {
			unsafe {
				(zerOS_eh_frame_hdr_end   as usize) -
				(zerOS_eh_frame_hdr_start as usize)
			}
		};
	}

	unsafe extern "C" {
		pub unsafe static zerOS_eh_frame_start: LinkerSym;
		pub unsafe static zerOS_eh_frame_end: LinkerSym;
	}
	lazy_static! {
		#[allow(non_upper_case_globals)]
		pub static ref zerOS_eh_frame_size: usize = {
			unsafe {
				(zerOS_eh_frame_end   as usize) -
				(zerOS_eh_frame_start as usize)
			}
		};
	}

	unsafe extern "C" {
		pub unsafe static zerOS_data_start: LinkerSym;
		pub unsafe static zerOS_data_end: LinkerSym;
	}
	lazy_static! {
		#[allow(non_upper_case_globals)]
		pub static ref zerOS_data_size: usize = {
			unsafe {
				(zerOS_data_end   as usize) -
				(zerOS_data_start as usize)
			}
		};
	}

	unsafe extern "C" {
		pub unsafe static zerOS_bss_start: LinkerSym;
		pub unsafe static zerOS_bss_end: LinkerSym;
	}
	lazy_static! {
		#[allow(non_upper_case_globals)]
		pub static ref zerOS_bss_size: usize = {
			unsafe {
				(zerOS_bss_end   as usize) -
				(zerOS_bss_start as usize)
			}
		};
	}
}

// --- SECTIONINFO AUTOGENERATED BY
// /home/axel/Documents/programmation/osdev/zerOS/scripts/gensectioninfo.py, END
// ---
