# zerOS

A hobby operating system from scratch.

## How to build zerOS ?

The build system used for zerOS is `cargo-make` [^1].

To build and test zerOS, the only things you need to do are :

  1. For now, modify `zerOS/kconfig.toml` if you want to change default config options.
  2. Then type `cargo make zerOS build profile=... arch=... cpu=...` to build zerOS. The following options are accepted :
    - `profile`: one of `dev`, `dev-lto`, `release` or `release-lto`
    - `arch`: for now, only `amd64` or an alias of it (e.g. `x64-64`, `x86_64`). Options likely to be the next ones I would choose to develop are `aarch64`, `riscv64`, and `x86`.
    - `cpu`: The optional target CPU/MCU, e.g. `skylake` or even `native`
  3. And finally, to run through QEMU, type `cargo make zerOS run arch=... cpu=... accel=...`, with `arch` and `cpu` being the same options as in the previous command, and `accel` being the optional name for a QEMU-supported accelerator (e.g. `kvm`). Note that debugging with GDB while running through an accelerator such as Linux's KVM is not supported/not possible at all.

[^1]: TODO: add a link to either the github or website of cargo-make
