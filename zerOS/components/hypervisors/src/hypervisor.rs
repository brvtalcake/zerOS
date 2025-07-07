use core::result::Result;

use overloadable::overloadable;

use crate::{arch::target::cpu::cpuid, kernel::sync::BasicMutex};

overloadable! {
	pub under_qemu as

	fn(cache_result: bool) -> Result<bool, core::array::TryFromSliceError>
	{
		// SAFETY: the two following variables are only accessed
		//         from this particular function, through a Mutex
		//         (see `MTX` below)
		static MTX: BasicMutex<(bool, bool)> = BasicMutex::new((false, false));

		fn get_caching_status() -> bool
		{
			unsafe {
				(*MTX.data_ptr()).0
			}
		}
		fn get_cached_value() -> bool
		{
			unsafe {
				(*MTX.data_ptr()).1
			}
		}
		fn set_caching_status(value: bool)
		{
			unsafe {
				(*MTX.data_ptr()).0 = value;
			}
		}
		fn set_cached_value(value: bool)
		{
			unsafe {
				(*MTX.data_ptr()).1 = value;
			}
		}

		#[allow(unused_variables)]
		let guard = MTX.lock();

		if cache_result && get_caching_status()
		{
			return Ok(get_cached_value());
		}

		const QEMU_HYPERV: [&[u8; 12]; 2] = [
			b"TCGTCGTCGTCG", b"KVMKVMKVM\0\0\0"
		];
		let cpuid_res = cpuid(0x40000000_u32);
		let hyperv: [u8; 12] = [
			cpuid_res.ebx.to_ne_bytes(),
			cpuid_res.ecx.to_ne_bytes(),
			cpuid_res.edx.to_ne_bytes(),
		].as_flattened()
			.try_into()?;
		let is_qemu = hyperv == *QEMU_HYPERV[0] || hyperv == *QEMU_HYPERV[1];
		if cache_result
		{
			set_cached_value(is_qemu);
			set_caching_status(true);
		}
		Ok(is_qemu)
	},

	fn() -> Result<bool, core::array::TryFromSliceError>
	{
		under_qemu(true)
	}
}
