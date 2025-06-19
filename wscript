from waflib.Configure import ConfigurationContext
from waflib.Options import OptionsContext
from waflib.Build import BuildContext

from scripts.get_valid_profiles import get_cargo_profiles

out_dir = "./build"

ZEROS_EMULATOR_CHOICES = ["qemu", "bochs"]

ZEROS_PROFILE_CHOICES = get_cargo_profiles("./zerOS/Cargo.toml")

ZEROS_ARCH_CHOICES = [
    "amd64",  # main
    "x86-64",
    "x86_64",
    "x86",  # main
    "i386",
    "i486",
    "i586",
    "i686",
    "aarch64",  # main
    "arm64",
    "arm32",  # main
    "arm",
    "riscv32",  # main
    "riscv64",  # main
    "powerpc32",  # main
    "ppc32",
    "powerpc64",  # main
    "ppc64",
    "ppc",
    "sparc32",  # main
    "sparc64",  # main
    "mips32",  # main
    "mips64",  # main
    "avr32",  # main
    "avr",
    "loongarch64",  # main
    "zarch",  # main
    "s390x",
]


def zerOS_arch_is_amd64(s: str) -> bool:
    return s in ["amd64", "x86-64", "x86_64"]


def zerOS_arch_is_x86(s: str) -> bool:
    return s in ["x86", "i386", "i486", "i586", "i686"]


def zerOS_arch_is_aarch64(s: str) -> bool:
    return s in ["aarch64", "arm64"]


def zerOS_arch_is_arm32(s: str) -> bool:
    return s in ["arm32", "arm"]


def zerOS_arch_is_riscv32(s: str) -> bool:
    return s in ["riscv32"]


def zerOS_arch_is_riscv64(s: str) -> bool:
    return s in ["riscv64"]


def zerOS_arch_is_ppc64(s: str) -> bool:
    return s in ["powerpc64", "ppc64", "ppc"]


def zerOS_arch_is_ppc32(s: str) -> bool:
    return s in ["powerpc32", "ppc32"]


def zerOS_arch_is_sparc64(s: str) -> bool:
    return s in ["sparc64"]


def zerOS_arch_is_sparc32(s: str) -> bool:
    return s in ["sparc32"]


def zerOS_arch_is_mips64(s: str) -> bool:
    return s in ["mips64"]


def zerOS_arch_is_mips32(s: str) -> bool:
    return s in ["mips32"]


def zerOS_arch_is_avr32(s: str) -> bool:
    return s in ["avr32", "avr"]


def zerOS_arch_is_loongarch64(s: str) -> bool:
    return s in ["loongarch64"]


def zerOS_arch_is_zarch(s: str) -> bool:
    return s in ["zarch", "s390x"]


def options(opts: OptionsContext):
    zeros_opts = opts.add_option_group("zerOS-specific options")
    zeros_opts.add_argument(
        "--arch",
        default="amd64",
        choices=ZEROS_ARCH_CHOICES,
        help="The architecture to `build` zerOS for, or to `run` zerOS on",
    )
    zeros_opts.add_argument(
        "--profile",
        choices=ZEROS_PROFILE_CHOICES,
        default="dev",
        help="The cargo profile to `build` zerOS with",
    )
    zeros_opts.add_argument(
        "--cpu",
        "--mcu",
        type=str,
        default="native",
        help="The CPU/MCU to `build` zerOS for, or to `run` zerOS on (note that for the run command, only some CPUs are valid)",
    )
    zeros_opts.add_argument(
        "--emulator",
        default="qemu",
        choices=ZEROS_EMULATOR_CHOICES,
        help="The emulator used to run zerOS",
    )
    zeros_opts.add_argument(
        "--gdb",
        action="store_true",
        default=False,
        help="Use to tell the emulator to act as a GDB remote target (to debug the running zerOS)",
    )
    zeros_opts.add_argument(
        "--kvm",
        action="store_true",
        default=False,
        help="Use KVM accelerator if possible",
    )


def configure(conf: ConfigurationContext):
    conf.load(["clang"])
    conf.find_program("cargo")
    conf.find_program("strip", mandatory=False)
    conf.find_program("eu-strip", var="EUSTRIP", mandatory=False)
    conf.find_program("objcopy", mandatory=False)
    conf.find_program("xorriso", mandatory=False)
    conf.find_program("bochs", mandatory=False)
    conf.find_program("gdb", mandatory=False)
    conf.find_program("rust-gdb", mandatory=False)
    conf.find_program("qemu-system-x86_64", mandatory=False)
    conf.find_program("qemu-system-aarch64", mandatory=False)
    conf.find_program("qemu-system-arm", mandatory=False)
    conf.find_program("qemu-system-avr", mandatory=False)
    conf.find_program("qemu-system-riscv32", mandatory=False)
    conf.find_program("qemu-system-riscv64", mandatory=False)
    conf.find_program("qemu-system-ppc", mandatory=False)
    conf.find_program("qemu-system-ppc64", mandatory=False)
    conf.find_program("qemu-system-sparc", mandatory=False)
    conf.find_program("qemu-system-sparc64", mandatory=False)
    conf.find_program("qemu-system-mips", mandatory=False)
    conf.find_program("qemu-system-mips64", mandatory=False)
    conf.find_program("qemu-system-mipsel", mandatory=False)
    conf.find_program("qemu-system-mips64el", mandatory=False)
    conf.find_program("qemu-system-loongarch64", mandatory=False)


def build(bld: BuildContext):
    print(bld)
