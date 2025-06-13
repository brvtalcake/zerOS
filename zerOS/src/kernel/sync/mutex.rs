use lock_api::{GuardSend, RawMutex};
use portable_atomic::{AtomicBool, Ordering};

/// TODO: also store cpu core id:
/// - see [this thread](https://stackoverflow.com/questions/22310028/is-there-an-x86-instruction-to-tell-which-core-the-instruction-is-being-run-on)
/// - see `IA32_TSC_AUX` MSR and `RDCPUID` instruction
/// TODO: maybe we should rather store some kind of thread ID
pub struct BasicMutexRaw
{
	locked: AtomicBool
}

unsafe impl Send for BasicMutexRaw {}
unsafe impl Sync for BasicMutexRaw {}

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
		debug_assert!(AtomicBool::is_always_lock_free());
		Self {
			locked: AtomicBool::new(false)
		}
	}

	#[inline(always)]
	fn relaxed_load(&self) -> bool
	{
		self.locked.load(Ordering::Relaxed)
	}
}

unsafe impl RawMutex for BasicMutexRaw
{
	type GuardMarker = GuardSend;

	const INIT: Self = Self::new();

	fn is_locked(&self) -> bool
	{
		self.locked.load(Ordering::Acquire)
	}

	fn lock(&self)
	{
		// Test and test-and-set
		loop
		{
			while self.relaxed_load()
			{
				core::hint::spin_loop();
			}

			if self.try_lock()
			{
				return;
			}
		}
	}

	fn try_lock(&self) -> bool
	{
		self.locked.swap(true, Ordering::AcqRel) == false
	}

	unsafe fn unlock(&self)
	{
		self.locked.store(false, Ordering::Release);
	}
}

pub type BasicMutex<T> = lock_api::Mutex<BasicMutexRaw, T>;
