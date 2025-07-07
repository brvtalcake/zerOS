#![no_std]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "limine")] {
        pub mod limine;
    } else if #[cfg(feature = "grub2")] {
        pub mod grub2;
    } else if #[cfg(feature = "uefi")] {
        compile_error!(
            stringify!(
                bootloader "uefi" is not implemented for now !
            )
        );
    } else {
        compile_error!(
            "unknown bootloader !"
        );
    }
}
