// *******************************************************************************
// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0
// *******************************************************************************

//!
//! Orch locks provide some basic locking mechanisms for the orchestration actions. This objects
//! are suited only for actions and their usage within orchestration as they account for use cases
//! that can happen there. They are not meant to be used in other contexts because they would most likely
//! not fir there.
//!

// needed until we use it in code
#![allow(dead_code)]

use ::core::ops::{Deref, DerefMut};

use kyron_foundation::{
    cell::{UnsafeCell, UnsafeCellExt},
    prelude::{CommonErrors, FoundationAtomicBool},
};

///
/// OrchTryLock provides a way to `try_lock` an object in orchestration. If this fails, call does not
/// block and leaves user with a problem of what to do in that case. This is useful because orchestration
/// actions are run "in a loop" so they do not really have concurrent access to the same object in normal
/// runs.
///
pub(crate) struct OrchTryLock<T> {
    data: UnsafeCell<T>,
    is_used: FoundationAtomicBool,
}

unsafe impl<T: Send> Send for OrchTryLock<T> {}
unsafe impl<T: Send> Sync for OrchTryLock<T> {}

impl<T> OrchTryLock<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            data: UnsafeCell::new(value),
            is_used: FoundationAtomicBool::new(false),
        }
    }

    /// Checks if the lock is currently held.
    pub(crate) fn is_locked(&self) -> bool {
        self.is_used.load(::core::sync::atomic::Ordering::SeqCst)
    }

    ///
    /// Tries to lock the object. If the lock is already held, it returns an error.
    ///
    /// # Errors
    ///    `CommonErrors::AlreadyDone` - If the lock is already held, it returns.
    ///
    pub(crate) fn try_lock(&self) -> Result<OrchTryLockGuard<'_, T>, CommonErrors> {
        if self
            .is_used
            .compare_exchange(
                false,
                true,
                ::core::sync::atomic::Ordering::SeqCst,
                ::core::sync::atomic::Ordering::SeqCst,
            )
            .is_err()
        {
            Err(CommonErrors::AlreadyDone)
        } else {
            Ok(OrchTryLockGuard { fake_mtx: self })
        }
    }
}

// Scoped guard that allows mutable access while it's held
pub(crate) struct OrchTryLockGuard<'a, T> {
    fake_mtx: &'a OrchTryLock<T>,
}

impl<T> OrchTryLockGuard<'_, T> {
    /// Access the underlying data immutably.
    pub(crate) fn with<R, F: FnOnce(&T) -> R>(&self, f: F) -> R {
        self.fake_mtx.data.with(|v| unsafe { f(&*v) })
    }

    /// Access the underlying data mutably.
    pub(crate) fn with_mut<R, F: FnOnce(&mut T) -> R>(&self, f: F) -> R {
        self.fake_mtx.data.with_mut(|v| unsafe { f(&mut *v) })
    }
}

impl<T> Deref for OrchTryLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.fake_mtx.data.as_ref_unchecked() }
    }
}

impl<T> DerefMut for OrchTryLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.fake_mtx.data.as_mut_unchecked() }
    }
}

impl<T> Drop for OrchTryLockGuard<'_, T> {
    fn drop(&mut self) {
        self.fake_mtx
            .is_used
            .store(false, ::core::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {

    use super::*;

    #[test]
    fn lock_drop_unlocks() {
        let obj = OrchTryLock::new(42);

        let lock = obj.try_lock().unwrap();

        assert!(obj.is_locked(), "Lock should be released after drop");
        assert!(obj.try_lock().is_err());
        drop(lock);

        assert!(!obj.is_locked(), "Lock should be released after drop");
        assert!(obj.try_lock().is_ok());
    }

    #[test]
    fn access_works() {
        let obj = OrchTryLock::new(42);

        let lock = obj.try_lock().unwrap();

        assert_eq!(42, lock.with(|v| *v));

        assert_eq!(
            32,
            lock.with_mut(|v| {
                *v = 32;
                *v
            })
        );
    }
}

#[cfg(test)]
#[cfg(loom)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn loom_try_lock() {
        loom::model(|| {
            let lock = Arc::new(OrchTryLock::new(42));
            let lock_l = lock.clone();

            let handle = loom::thread::spawn(move || {
                let guard_res = lock.try_lock();

                match guard_res {
                    Ok(guard) => {
                        guard.with_mut(|v| *v = 43);

                        true
                    },
                    Err(_) => false,
                }
            });

            {
                let lock_res = lock_l.try_lock();

                match lock_res {
                    Ok(guard) => {
                        guard.with_mut(|v| *v = 44);
                        true
                    },
                    Err(_) => false,
                };
            }

            let _ = handle.join().unwrap();

            // We dont assert anything, we are fine with running it by loom and breaking when concurrent access to UnsafeCell happens
        });
    }
}
