use core::result::Result;

use overloadable::overloadable;

use crate::kernel::{
	cpuid,
	sync::{Mutex, MutexGuard}
};

overloadable! {
	pub under_qemu as

	fn(cache_result: bool) -> Result<bool, core::array::TryFromSliceError>
	{
		// SAFETY: the two following variables are only accessed
		//         from this particular function, through a Mutex
		//         (see `MTX` below)
		static mut HAS_BEEN_CACHED: bool = false;
		static mut CACHED_VALUE   : bool = false;
		static MTX: Mutex = Mutex::new();

		#[allow(unused_variables)]
		let guard = MutexGuard::from(&MTX);

		if cache_result && unsafe { HAS_BEEN_CACHED }
		{
			return Ok(unsafe { CACHED_VALUE });
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
			unsafe {
				CACHED_VALUE = is_qemu;
				HAS_BEEN_CACHED = true;
			}
		}
		Ok(is_qemu)
	},

	fn() -> Result<bool, core::array::TryFromSliceError>
	{
		under_qemu(true)
	}
}
