use core::ffi::c_void;

use crate::kernel::linker::map::*;

pub fn in_section(secname: &str, addr: *const c_void) -> Option<bool>
{
    type TupleType = (&'static str, LinkerSym, LinkerSym);
    #[rustfmt::skip]
    let TUPLES: [TupleType; zerOS_section_count] = [
        ("text", unsafe { zerOS_text_start }, unsafe { zerOS_text_end }),
        ("bootcode", unsafe { zerOS_bootcode_start }, unsafe { zerOS_bootcode_end }),
        ("ctors_init_array", unsafe { zerOS_ctors_init_array_start }, unsafe { zerOS_ctors_init_array_end }),
        ("rodata", unsafe { zerOS_rodata_start }, unsafe { zerOS_rodata_end }),
        ("data", unsafe { zerOS_data_start }, unsafe { zerOS_data_end }),
        ("bss", unsafe { zerOS_bss_start }, unsafe { zerOS_bss_end }),
    ];
    let found = TUPLES
        .iter()
        .find(|elem| elem.0.eq_ignore_ascii_case(secname))?;
    Some((found.1 as usize) <= (addr as usize) && (addr as usize) <= (found.2 as usize))
}
