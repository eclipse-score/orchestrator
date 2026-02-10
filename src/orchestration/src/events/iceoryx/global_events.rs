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

use kyron_foundation::prelude::trace;

///////////////////////////////////////////////////////////////////////////////////////////////////
/// IMPORTANT: This is temporary solution for iceoryx integration. This will be re-written later.
///////////////////////////////////////////////////////////////////////////////////////////////////
use crate::actions::action::ActionResult;
use crate::events::event_traits::{IpcProvider, ListenerTrait, NotifierTrait};
use crate::events::iceoryx::event::Event;
use ::core::future::Future;
use ::core::pin::Pin;
use ::core::task::{Context, Poll};

/// GlobalEvents implements the IpcProvider trait
pub struct GlobalEvents;

impl GlobalEvents {
    pub fn new() -> Self {
        Self {} // Nothing to initialize now.
    }
}

impl Default for GlobalEvents {
    fn default() -> Self {
        Self::new()
    }
}

// Implementation of IpcProvider trait
impl IpcProvider for GlobalEvents {
    fn new() -> Self {
        GlobalEvents::new()
    }

    fn get_notifier(&mut self, event_name: &str) -> Option<impl NotifierTrait + Send + 'static> {
        Event::get_instance().lock().unwrap().create_notifier(event_name);
        Some(IpcNotifier {
            notifier: event_name.to_string(),
        })
    }

    fn get_listener(&mut self, event_name: &str) -> Option<impl ListenerTrait + Send + 'static> {
        let listener = Event::get_instance().lock().unwrap().create_listener(event_name);
        Some(IpcListener { listener })
    }
}

// IpcNotifier is a notifier that triggers an IPC event
pub struct IpcNotifier {
    notifier: String,
}
impl IpcNotifier {
    async fn trigger_async(event_name: String) -> ActionResult {
        let result = Event::get_instance().lock().unwrap().trigger_event(event_name.as_str());
        trace!("GlobalNotifier: triggered event: {}", event_name);
        result
    }
}
impl NotifierTrait for IpcNotifier {
    #[allow(clippy::manual_async_fn)]
    fn notify(&self, _value: u32) -> impl Future<Output = ActionResult> + Send + 'static {
        Self::trigger_async(self.notifier.clone())
    }

    // Yes, it's copy-paste, but it doesn't clone the string unnecessarily.
    fn notify_sync(&self, _value: u32) -> ActionResult {
        let result = Event::get_instance().lock().unwrap().trigger_event(&self.notifier);
        result
    }
}

// IpcListener is a listener that waits for an IPC event to be triggered
pub struct IpcListener {
    listener: usize,
}
impl IpcListener {
    async fn execute_impl(listener: usize) -> ActionResult {
        struct IpcListenerInner {
            listener: usize,
        }
        impl Future for IpcListenerInner {
            type Output = ();
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let waker_clone = cx.waker().clone();
                let event_received = Event::get_instance()
                    .lock()
                    .unwrap()
                    .wake_on_event(self.listener, waker_clone);
                if event_received {
                    trace!("GlobalListener: received event for listener: {}", self.listener);
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        }
        IpcListenerInner { listener }.await;

        Ok(())
    }
}
impl ListenerTrait for IpcListener {
    fn next(&mut self) -> impl Future<Output = ActionResult> + Send + 'static {
        Self::execute_impl(self.listener)
    }
}
