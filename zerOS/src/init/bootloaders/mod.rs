use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(bootloader = "limine")] {
        pub mod limine;
    } else if #[cfg(bootloader = "grub2")] {
        pub mod grub;
    } else if #[cfg(bootloader = "uefi")] {
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
