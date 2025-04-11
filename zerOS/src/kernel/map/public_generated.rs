pub use super::__generated::__linker_symbols::zerOS_text_start;
pub use super::__generated::__linker_symbols::zerOS_text_end;
pub use super::__generated::__linker_symbols::zerOS_text_size;

pub use super::__generated::__linker_symbols::zerOS_bootcode_start;
pub use super::__generated::__linker_symbols::zerOS_bootcode_end;
pub use super::__generated::__linker_symbols::zerOS_bootcode_size;

pub use super::__generated::__linker_symbols::zerOS_rodata_start;
pub use super::__generated::__linker_symbols::zerOS_rodata_end;
pub use super::__generated::__linker_symbols::zerOS_rodata_size;

pub use super::__generated::__linker_symbols::zerOS_data_start;
pub use super::__generated::__linker_symbols::zerOS_data_end;
pub use super::__generated::__linker_symbols::zerOS_data_size;

pub use super::__generated::__linker_symbols::zerOS_bss_start;
pub use super::__generated::__linker_symbols::zerOS_bss_end;
pub use super::__generated::__linker_symbols::zerOS_bss_size;


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
