use core::ffi::c_void;

use crate::map::*;

pub fn in_section(secname: &str, addr: *const c_void) -> Option<bool>
{
	let TUPLES: [_; zerOS_section_count] = [
		(
			"text",
			&raw const zerOS_text_start,
			&raw const zerOS_text_end
		),
		(
			"bootcode",
			&raw const zerOS_bootcode_start,
			&raw const zerOS_bootcode_end
		),
		(
			"ctors_init_array",
			&raw const zerOS_ctors_init_array_start,
			&raw const zerOS_ctors_init_array_end
		),
		(
			"rodata",
			&raw const zerOS_rodata_start,
			&raw const zerOS_rodata_end
		),
		(
			"eh_frame",
			&raw const zerOS_eh_frame_start,
			&raw const zerOS_eh_frame_end
		),
		(
			"eh_frame_hdr",
			&raw const zerOS_eh_frame_hdr_start,
			&raw const zerOS_eh_frame_hdr_end
		),
		(
			"data",
			&raw const zerOS_data_start,
			&raw const zerOS_data_end
		),
		("bss", &raw const zerOS_bss_start, &raw const zerOS_bss_end)
	];
	let found = TUPLES
		.iter()
		.find(|elem| elem.0.eq_ignore_ascii_case(secname))?;
	Some(found.1.addr() <= addr.addr() && addr.addr() <= found.2.addr())
}
