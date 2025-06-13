mod mutex;
mod rwlock;

pub use mutex::{BasicMutex, BasicMutexRaw};
pub use rwlock::{BasicRwLock, BasicRwLockRaw};
