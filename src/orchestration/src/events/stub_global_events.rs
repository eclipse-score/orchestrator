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

use crate::actions::action::ActionResult;
use crate::events::event_traits::{IpcProvider, ListenerTrait, NotifierTrait};
use ::core::future::Future;
use kyron_foundation::prelude::*;

/// StubGlobalEvents to enable compilation when iceoryx IPC is not enabled.
pub struct StubGlobalEvents;

impl StubGlobalEvents {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for StubGlobalEvents {
    fn default() -> Self {
        Self::new()
    }
}

// Stub implementation of IpcProvider trait
impl IpcProvider for StubGlobalEvents {
    fn new() -> Self {
        warn!("This is stub implementation. Global events will not work!");
        StubGlobalEvents::new()
    }

    fn get_notifier(&mut self, _event_name: &str) -> Option<impl NotifierTrait + Send + 'static> {
        warn!("This is stub implementation. Global events will not work!");
        None::<StubIpcNotifier>
    }

    fn get_listener(&mut self, _event_name: &str) -> Option<impl ListenerTrait + Send + 'static> {
        warn!("This is stub implementation. Global events will not work!");
        None::<StubIpcListener>
    }
}

/// StubIpcNotifier to enable compilation when iceoryx IPC is not enabled.
pub struct StubIpcNotifier;
impl NotifierTrait for StubIpcNotifier {
    #[allow(clippy::manual_async_fn)]
    fn notify(&self, _value: u32) -> impl Future<Output = ActionResult> + Send + 'static {
        warn!("This is stub implementation. Global events will not work!");
        async { Ok(()) }
    }

    fn notify_sync(&self, _value: u32) -> crate::prelude::ActionResult {
        warn!("This is stub implementation. Global events will not work!");
        Ok(())
    }
}

/// StubIpcListener to enable compilation when iceoryx IPC is not enabled.
pub struct StubIpcListener;
impl ListenerTrait for StubIpcListener {
    fn next(&mut self) -> impl Future<Output = ActionResult> + Send + 'static {
        warn!("This is stub implementation. Global events will not work!");
        async { Ok(()) }
    }
}
