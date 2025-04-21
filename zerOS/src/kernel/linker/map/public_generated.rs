pub use super::__generated::__linker_symbols::{zerOS_bootcode_end,
                                               zerOS_bootcode_size,
                                               zerOS_bootcode_start,
                                               zerOS_bss_end,
                                               zerOS_bss_size,
                                               zerOS_bss_start,
                                               zerOS_ctors_init_array_end,
                                               zerOS_ctors_init_array_size,
                                               zerOS_ctors_init_array_start,
                                               zerOS_data_end,
                                               zerOS_data_size,
                                               zerOS_data_start,
                                               zerOS_kernel_end,
                                               zerOS_kernel_start,
                                               zerOS_rodata_end,
                                               zerOS_rodata_size,
                                               zerOS_rodata_start,
                                               zerOS_section_count,
                                               zerOS_text_end,
                                               zerOS_text_size,
                                               zerOS_text_start};

#[cfg(test)]
mod tests
{
	use super::zerOS_bss_size;

	#[test]
	fn compiles()
	{
		let _test = *zerOS_bss_size;
	}
}
