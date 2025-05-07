use core::sync::atomic::{AtomicBool, Ordering};

use lock_api::{GuardSend, RawMutex};

/// TODO: also store cpu core id:
/// - see [this thread](https://stackoverflow.com/questions/22310028/is-there-an-x86-instruction-to-tell-which-core-the-instruction-is-being-run-on)
/// - see `IA32_TSC_AUX` MSR and `RDCPUID` instruction
pub struct BasicMutexRaw
{
	flag: AtomicBool
}

impl Default for BasicMutexRaw
{
	fn default() -> Self
	{
		Self::new()
	}
}

impl BasicMutexRaw
{
	pub const fn new() -> Self
	{
		Self {
			flag: AtomicBool::new(false)
		}
	}
}

unsafe impl RawMutex for BasicMutexRaw
{
	type GuardMarker = GuardSend;

	const INIT: Self = Self::new();

	fn is_locked(&self) -> bool
	{
		self.flag.load(Ordering::Acquire)
	}

	fn lock(&self)
	{
		while self
			.flag
			.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
			.is_err()
		{
			core::hint::spin_loop();
		}
	}

	fn try_lock(&self) -> bool
	{
		match self
			.flag
			.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
		{
			Ok(false) => true,
			Ok(true) => unreachable!(),
			Err(true) => false,
			Err(false) => unreachable!()
		}
	}

	unsafe fn unlock(&self)
	{
		self.flag.store(false, Ordering::Release);
	}
}

pub type BasicMutex<T> = lock_api::Mutex<BasicMutexRaw, T>;
