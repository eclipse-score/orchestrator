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

#![allow(dead_code)]

use ::core::{
    sync::atomic::{AtomicI32, Ordering},
    time::Duration,
};
use libc::{sigaction, sighandler_t, SIGINT, SIGTERM};

pub struct SignalHandler {
    signal: AtomicI32,
}

/// This struct provides a way to handle signals SIGINT and SIGTERM.
impl SignalHandler {
    /// Returns a reference to the singleton instance of `SignalHandler`.
    pub fn get_instance() -> &'static Self {
        &SIGNAL_HANDLER
    }

    /// Registers signal handlers for SIGINT and SIGTERM.
    pub unsafe fn register_signal_handlers(&self) {
        let mut action: sigaction = core::mem::zeroed();
        action.sa_sigaction = handler as sighandler_t;

        sigaction(SIGINT, &action, core::ptr::null_mut());
        sigaction(SIGTERM, &action, core::ptr::null_mut());
    }

    /// Returns the current signal value.
    pub fn is_signal_received(&self) -> i32 {
        self.signal.load(Ordering::SeqCst)
    }

    /// Waits until a signal is received.
    pub fn wait_until_signal_received(&self) -> i32 {
        let mut received_signal;

        loop {
            // Check if a signal has been received
            received_signal = self.signal.load(Ordering::SeqCst);
            if received_signal != 0 {
                break;
            }

            // Sleep for a short duration to avoid busy waiting
            std::thread::sleep(Duration::from_millis(20));
        }

        received_signal
    }
}

static SIGNAL_HANDLER: SignalHandler = SignalHandler {
    signal: AtomicI32::new(0),
};

extern "C" fn handler(sig: i32) {
    SIGNAL_HANDLER.signal.store(sig, Ordering::SeqCst);
}
