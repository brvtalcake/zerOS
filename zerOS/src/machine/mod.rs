use core::arch::{asm, global_asm};

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    hcf();
}

fn _hcf() -> ! {
    loop {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            asm!("hlt");
            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            asm!("wfi");
            #[cfg(target_arch = "loongarch64")]
            asm!("idle 0");
        }
    }
}

pub fn hcf() -> !
{
    loop {
        /*
         * expected values for `target_arch` are:
         *      `aarch64`, `amdgpu`, `arm`, `arm64ec`, `avr`, `bpf`, `csky`,
         *      `hexagon`, `loongarch64`, `m68k`, `mips`, `mips32r6`, `mips64`,
         *      `mips64r6`, `msp430`, `nvptx64`, `powerpc`, `powerpc64`, `riscv32`,
         *      `riscv64`, `s390x`, `sparc`, `sparc64`, `wasm32`, `wasm64`, `x86`,
         *      `x86_64`, and `xtensa`
         */
        unsafe {
            #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
            {
                todo!();
            }
            #[cfg(any(target_arch = "powerpc", target_arch = "powerpc64"))]
            {
                todo!();
            }
            #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
            {
                todo!();
            }
            #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
            {
                x86::irq::disable();
                x86::halt();
            }
        }
    }
}