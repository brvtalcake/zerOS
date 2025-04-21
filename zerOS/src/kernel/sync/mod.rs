use core::sync::atomic::{AtomicBool, Ordering};

pub struct Mutex
{
    flag: AtomicBool,
}

impl Default for Mutex {
    fn default() -> Self {
        Self::new()
    }
}

impl Mutex
{
    pub const fn new() -> Self
    {
        Self { flag: AtomicBool::new(false) }
    }

    pub fn lock(&self)
    {
        while self.flag.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err()
        { }
    }

    pub fn unlock(&self)
    {
        self.flag.store(false, Ordering::Release);
    }

    pub fn locked(&self) -> bool
    {
        self.flag.load(Ordering::Acquire)
    }
}

pub struct MutexGuard<'a>
{
    mtx: &'a Mutex,
}

impl<'a> From<&'a Mutex> for MutexGuard<'a>
{
    fn from(value: &'a Mutex) -> Self
    {
        value.lock();
        Self { mtx: value }
    }
}

impl<'a> Drop for MutexGuard<'a>
{
    fn drop(&mut self)
    {
        self.mtx.unlock();
    }
}