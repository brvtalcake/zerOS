use cfg_aliases::cfg_aliases;

pub fn generate_config_arch_aliases()
{
	cfg_aliases! {
		x86_alike: { any(target_arch = "x86", target_arch = "x86_64") },
		avr_alike: { target_arch = "avr" },
		sparc_alike: { any(target_arch = "sparc", target_arch = "sparc64") },
		loongarch_alike: { target_arch = "loongarch64" },
		mips_alike: { any(
			target_arch = "mips",
			target_arch = "mips64",
			target_arch = "mips32r6",
			target_arch = "mips64r6") },
		ppc_alike: { any(target_arch = "powerpc", target_arch = "powerpc64") },
		riscv_alike: { any(target_arch = "riscv32", target_arch = "riscv64") },
		arm_alike: { any(target_arch = "aarch64", target_arch = "arm", target_arch = "arm64ec") },
		zarch_alike: { any(target_arch = "s390", target_arch = "s390x") }
		// TODO: IA64, Alpha DEC, SuperH, OpenRISC (?), C-Sky, HPPA (?)
	};
}

#[macro_export]
macro_rules! custom_kcfg {
	($cfg:ident : $type:ty = $parsed:expr) => {
		to_cargo!("rustc-cfg" => format!("{}=\"{}\"", stringify!($cfg), $parsed));
		let mut cfgstr = String::from(format!("cfg({}, values(", stringify!($cfg)));
		cfgstr += format!("\"{}\"", <$type>::VARIANTS[0]).as_str();
		for authorized in <$type>::VARIANTS.iter().skip(1)
		{
			cfgstr += format!(",\"{}\"", authorized).as_str();
		}
		cfgstr += "))";
		to_cargo!("rustc-check-cfg" => cfgstr);
	};
}