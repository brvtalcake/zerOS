use lock_api::{GuardSend, RawMutex, RawRwLock, RwLock};

use super::BasicMutex;

#[repr(u8)]
enum RwLockState
{
	Unlocked,
	Reading,
	Writing
}

struct BasicRwLockRawInner
{
	readers: usize
}

pub struct BasicRwLockRaw
{
	inner: BasicMutex<BasicRwLockRawInner>
}

unsafe impl Sync for BasicRwLockRaw {}
unsafe impl Send for BasicRwLockRaw {}

unsafe impl RawRwLock for BasicRwLockRaw
{
	type GuardMarker = GuardSend;

	const INIT: Self = Self {
		inner: BasicMutex::new(BasicRwLockRawInner { readers: 0 })
	};

	fn is_locked(&self) -> bool
	{
		self.inner.try_lock().is_none_or(|inner| inner.readers != 0)
	}

	fn is_locked_exclusive(&self) -> bool
	{
		self.inner.try_lock().is_none()
	}

	fn try_lock_exclusive(&self) -> bool
	{
		// SAFETY: completely safe since calling `.raw()` is only `unsafe` when
		// unlocking the mutex while still holding a reference to a `MutexGuard` and we
		// precisely don't do that
		if unsafe { self.inner.raw().try_lock() }
		{
			// SAFETY: we just locked the mutex
			if unsafe { self.inner.data_ptr().as_ref_unchecked() }.readers == 0
			{
				return true;
			}
			// SAFETY: we just locked the mutex
			unsafe {
				self.inner.raw().unlock();
			}
		}
		false
	}

	fn try_lock_shared(&self) -> bool
	{
		// SAFETY: completely safe since calling `.raw()` is only `unsafe` when
		// unlocking the mutex while still holding a reference to a `MutexGuard` and we
		// precisely don't do that
		if unsafe { self.inner.raw().try_lock() }
		{
			// we locked the mutex so we are garanteed that no writing thread holds the
			// RwLock

			// SAFETY: we just locked the mutex
			unsafe {
				self.inner.data_ptr().as_mut_unchecked().readers += 1;
				self.inner.raw().unlock();
			}
			true
		}
		else
		{
			false
		}
	}

	fn lock_exclusive(&self)
	{
		while !self.try_lock_exclusive()
		{
			core::hint::spin_loop();
		}
	}

	fn lock_shared(&self)
	{
		while !self.try_lock_shared()
		{
			core::hint::spin_loop();
		}
	}

	unsafe fn unlock_exclusive(&self)
	{
		// SAFETY: the mutex is locked at the end of `.try_lock_exclusive()`
		unsafe {
			self.inner.raw().unlock();
		}
	}

	unsafe fn unlock_shared(&self)
	{
		self.inner.lock().readers -= 1;
	}
}

pub type BasicRwLock<T> = RwLock<BasicRwLockRaw, T>;
