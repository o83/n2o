// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;
use std::marker;
use std::ops::Deref;
use super::poison::{self, TryLockError, TryLockResult, LockResult};
use super::sys_mutex as sys;

/// A re-entrant mutual exclusion
///
/// This mutex will block *other* threads waiting for the lock to become
/// available. The thread which has already locked the mutex can lock it
/// multiple times without blocking, preventing a common source of deadlocks.
pub struct ReentrantMutex<T> {
    inner: Box<sys::ReentrantMutex>,
    poison: poison::Flag,
    data: T,
}

unsafe impl<T: Send> Send for ReentrantMutex<T> {}
unsafe impl<T: Send> Sync for ReentrantMutex<T> {}


/// An RAII implementation of a "scoped lock" of a mutex. When this structure is
/// dropped (falls out of scope), the lock will be unlocked.
///
/// The data protected by the mutex can be accessed through this guard via its
/// Deref implementation.
///
/// # Mutability
///
/// Unlike `MutexGuard`, `ReentrantMutexGuard` does not implement `DerefMut`,
/// because implementation of the trait would violate Rust’s reference aliasing
/// rules. Use interior mutability (usually `RefCell`) in order to mutate the
/// guarded data.
#[must_use]
pub struct ReentrantMutexGuard<'a, T: 'a> {
    // funny underscores due to how Deref currently works (it disregards field
    // privacy).
    __lock: &'a ReentrantMutex<T>,
    __poison: poison::Guard,
}

impl<'a, T> !marker::Send for ReentrantMutexGuard<'a, T> {}


impl<T> ReentrantMutex<T> {
    /// Creates a new reentrant mutex in an unlocked state.
    pub fn new(t: T) -> ReentrantMutex<T> {
        unsafe {
            let mut mutex = ReentrantMutex {
                inner: box sys::ReentrantMutex::uninitialized(),
                poison: poison::Flag::new(),
                data: t,
            };
            mutex.inner.init();
            mutex
        }
    }

    /// Acquires a mutex, blocking the current thread until it is able to do so.
    ///
    /// This function will block the caller until it is available to acquire the mutex.
    /// Upon returning, the thread is the only thread with the mutex held. When the thread
    /// calling this method already holds the lock, the call shall succeed without
    /// blocking.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return failure if the mutex would otherwise be
    /// acquired.
    pub fn lock(&self) -> LockResult<ReentrantMutexGuard<T>> {
        unsafe { self.inner.lock() }
        ReentrantMutexGuard::new(&self)
    }

    /// Attempts to acquire this lock.
    ///
    /// If the lock could not be acquired at this time, then `Err` is returned.
    /// Otherwise, an RAII guard is returned.
    ///
    /// This function does not block.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return failure if the mutex would otherwise be
    /// acquired.
    pub fn try_lock(&self) -> TryLockResult<ReentrantMutexGuard<T>> {
        if unsafe { self.inner.try_lock() } {
            Ok(ReentrantMutexGuard::new(&self)?)
        } else {
            Err(TryLockError::WouldBlock)
        }
    }
}

impl<T> Drop for ReentrantMutex<T> {
    fn drop(&mut self) {
        // This is actually safe b/c we know that there is no further usage of
        // this mutex (it's up to the user to arrange for a mutex to get
        // dropped, that's not our job)
        unsafe { self.inner.destroy() }
    }
}

impl<T: fmt::Debug + 'static> fmt::Debug for ReentrantMutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_lock() {
            Ok(guard) => write!(f, "ReentrantMutex {{ data: {:?} }}", &*guard),
            Err(TryLockError::Poisoned(err)) => {
                write!(f,
                       "ReentrantMutex {{ data: Poisoned({:?}) }}",
                       &**err.get_ref())
            }
            Err(TryLockError::WouldBlock) => write!(f, "ReentrantMutex {{ <locked> }}"),
        }
    }
}

impl<'mutex, T> ReentrantMutexGuard<'mutex, T> {
    fn new(lock: &'mutex ReentrantMutex<T>) -> LockResult<ReentrantMutexGuard<'mutex, T>> {
        poison::map_result(lock.poison.borrow(), |guard| {
            ReentrantMutexGuard {
                __lock: lock,
                __poison: guard,
            }
        })
    }
}

impl<'mutex, T> Deref for ReentrantMutexGuard<'mutex, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.__lock.data
    }
}

impl<'a, T> Drop for ReentrantMutexGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.__lock.poison.done(&self.__poison);
            self.__lock.inner.unlock();
        }
    }
}
