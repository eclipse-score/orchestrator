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
use ::core::future::Future;

/// NotifierTrait defines the interface for a notifier that can notify listeners with a value.
pub trait NotifierTrait {
    fn notify(&self, value: u32) -> impl Future<Output = ActionResult> + Send + 'static;
    fn notify_sync(&self, value: u32) -> ActionResult;
}

/// ListenerTrait defines the interface for a listener that can listen for notifications and return an action result.
pub trait ListenerTrait {
    fn next(&mut self) -> impl Future<Output = ActionResult> + Send + 'static;
}

/// IpcProvider defines the interface for an IPC provider that can provide notifiers and listeners based on a tag.
pub trait IpcProvider {
    /// Creates a new instance of the IPC provider.
    fn new() -> Self;
    /// Returns a notifier for the given tag.
    fn get_notifier(&mut self, event_name: &str) -> Option<impl NotifierTrait + Send + 'static>;
    /// Returns a listener for the given tag.
    fn get_listener(&mut self, event_name: &str) -> Option<impl ListenerTrait + Send + 'static>;
}
