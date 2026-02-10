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
use ::core::future::Future;
use std::sync::Arc;

use kyron::channels::spmc_broadcast::*;

use crate::common::tag::Tag;
use crate::events::event_traits::{ListenerTrait, NotifierTrait};
use crate::{
    actions::action::{ActionExecError, ActionResult},
    core::orch_locks::OrchTryLock,
};
use kyron_foundation::prelude::*;

const MAX_NUM_OF_EVENTS: usize = 8;

pub struct LocalEvent {
    id: Tag,
    sender: Option<Sender<u32, MAX_NUM_OF_EVENTS>>,
    receiver: Receiver<u32, MAX_NUM_OF_EVENTS>,
}

impl LocalEvent {
    pub fn new(id: Tag) -> Self {
        let (s, r) = create_channel::<u32, MAX_NUM_OF_EVENTS>(8);
        Self {
            id,
            sender: Some(s),
            receiver: r,
        }
    }

    pub fn get_notifier(&mut self) -> Option<LocalNotifier> {
        self.sender.take().map(|v| LocalNotifier {
            id: self.id,
            sender: Arc::new(v),
        })
    }

    pub fn get_listener(&mut self) -> Option<LocalListener> {
        self.receiver.try_clone().map(|v| LocalListener {
            id: self.id,
            receiver: Arc::new(OrchTryLock::new(v)),
        })
    }
}

pub struct LocalNotifier {
    id: Tag,
    sender: Arc<Sender<u32, MAX_NUM_OF_EVENTS>>,
}

impl LocalNotifier {
    fn exec_sync(notifier: Arc<Sender<u32, MAX_NUM_OF_EVENTS>>, value: u32, id: Tag) -> ActionResult {
        debug!("LocalNotifier({:?}): Notifier sending value: {}", id, value);
        notifier.send(&value).map_err(|e| {
            error!(
                "LocalNotifier({:?}): Failed to send value: {} with error {:?}",
                id, value, e
            );
            ActionExecError::NonRecoverableFailure
        })
    }

    async fn exec_async(notifier: Arc<Sender<u32, MAX_NUM_OF_EVENTS>>, value: u32, id: Tag) -> ActionResult {
        Self::exec_sync(notifier, value, id)
    }
}

impl NotifierTrait for LocalNotifier {
    #[allow(clippy::manual_async_fn)]
    fn notify(&self, value: u32) -> impl Future<Output = ActionResult> + Send + 'static {
        Self::exec_async(self.sender.clone(), value, self.id)
    }

    fn notify_sync(&self, value: u32) -> ActionResult {
        Self::exec_sync(self.sender.clone(), value, self.id)
    }
}

pub struct LocalListener {
    id: Tag,
    receiver: Arc<OrchTryLock<Receiver<u32, MAX_NUM_OF_EVENTS>>>, // Arc used here to "share between futures, not between actions"
}

impl LocalListener {
    async fn execute_impl(listener: Arc<OrchTryLock<Receiver<u32, MAX_NUM_OF_EVENTS>>>, id: Tag) -> ActionResult {
        match listener.try_lock() {
            Ok(mut receiver) => {
                if (receiver.recv().await).is_some() {
                    debug!("LocalSync({:?}): Listener received an event", id);
                    Ok(())
                } else {
                    error!("LocalSync({:?}): Listener lost its notifier!", id);
                    Err(ActionExecError::NonRecoverableFailure)
                }
            },
            Err(_) => {
                error!("LocalSync({:?}): Listener is already locked, fatal failure!", id);
                Err(ActionExecError::NonRecoverableFailure)
            },
        }
    }
}

impl ListenerTrait for LocalListener {
    fn next(&mut self) -> impl Future<Output = ActionResult> + Send + 'static {
        Self::execute_impl(self.receiver.clone(), self.id)
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use crate::testing::OrchTestingPoller;

    use super::*;

    #[test]
    fn local_listener_success() {
        let mut event = LocalEvent::new("test_event".into());
        let notifier = event.get_notifier().expect("Notifier should be available");
        let mut listener = event.get_listener().expect("Listener should be available");

        // Notify with a value and then listen for it
        assert!(OrchTestingPoller::block_on(async move {
            let notify_result = notifier.notify(42).await;
            assert!(notify_result.is_ok(), "Notify should succeed");

            let listen_result = listener.next().await;
            assert!(listen_result.is_ok(), "Listener should receive event");
        })
        .is_some());
    }

    #[test]
    fn local_listener_lost_notifier() {
        let mut event = LocalEvent::new("test_event".into());
        let _notifier = event.get_notifier().expect("Notifier should be available");
        let mut listener = event.get_listener().expect("Listener should be available");

        // Drop the notifier before listening
        drop(_notifier);

        assert!(OrchTestingPoller::block_on(async move {
            let listen_result = listener.next().await;
            assert!(listen_result.is_err(), "Listener should fail if notifier is gone");
        })
        .is_some());
    }

    #[test]
    fn local_listener_already_locked() {
        let mut event = LocalEvent::new("test_event".into());
        let _notifier = event.get_notifier().expect("Notifier should be available");
        let mut listener = event.get_listener().expect("Listener should be available");

        // Simulate lock contention by locking manually
        let receiver_arc = listener.receiver.clone();
        let _lock = receiver_arc.try_lock().expect("Should be able to lock");

        assert!(OrchTestingPoller::block_on(async move {
            let listen_result = listener.next().await;
            assert!(listen_result.is_err(), "Listener should fail if already locked");
        })
        .is_some());
    }
}
