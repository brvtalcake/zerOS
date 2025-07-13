#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(ptr_as_ref_unchecked)]

mod mutex;
mod rwlock;

pub use mutex::{BasicMutex, BasicMutexRaw};
pub use rwlock::{BasicRwLock, BasicRwLockRaw};
