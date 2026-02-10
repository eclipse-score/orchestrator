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
use ::core::cell::RefCell;
use core::time::Duration;
use std::rc::Rc;

use crate::common::DesignConfig;
use crate::events::event_traits::{IpcProvider, NotifierTrait};
use crate::events::timer_events::TimerEvent;
use crate::events::GlobalEventProvider;
use crate::prelude::ActionResult;
use crate::{
    actions::{sync::Sync, trigger::Trigger},
    common::tag::AsTagTrait,
};
use kyron_foundation::prelude::vector_extension::VectorExtension;
use kyron_foundation::prelude::*;

use crate::{actions::action::ActionTrait, common::tag::Tag, events::local_events::LocalEvent};

pub const DEFAULT_EVENTS_CAPACITY: usize = 256;

#[derive(Copy, Clone, Debug, PartialEq)]
enum EventType {
    /// Event that is process local
    Local,

    /// Even that is cross processes
    Global,

    /// Timer event
    Timer,
}

///
/// Provides real events into design and allows to specify which design events should map to it.
///
pub struct EventsProvider<GlobalProvider = GlobalEventProvider>
where
    GlobalProvider: IpcProvider,
{
    events: Vec<DeploymentEventInfo>,
    local_event_next_id: u64,
    timer_event_next_id: u64,
    pub(crate) ipc: Rc<RefCell<GlobalProvider>>,
}

impl<GlobalProvider: IpcProvider + 'static> Default for EventsProvider<GlobalProvider> {
    fn default() -> Self {
        Self::new()
    }
}

impl<GlobalProvider: IpcProvider + 'static> EventsProvider<GlobalProvider> {
    pub fn new() -> Self {
        Self {
            events: Vec::new_in_global(DEFAULT_EVENTS_CAPACITY),
            local_event_next_id: 0,
            ipc: Rc::new(RefCell::new(GlobalProvider::new())),
            timer_event_next_id: 0,
        }
    }

    /// Deployment time event specification
    /// This let integrator register new event and specify whether it's local or global and which design events should map to it.
    pub(crate) fn specify_global_event(
        &mut self,
        system_event: &str,
        events_to_bind: &[Tag],
    ) -> Result<EventCreator, CommonErrors> {
        let ipc_c = Rc::clone(&self.ipc);

        self.specify_event(system_event, EventType::Global, events_to_bind, |evt_name, _| {
            GlobalEventCreator {
                system_event_name: evt_name.to_string(),
                global_provider: ipc_c,
            }
        })
    }

    pub(crate) fn specify_local_event(&mut self, events_to_bind: &[Tag]) -> Result<EventCreator, CommonErrors> {
        let name = format!("local_event_{}", self.local_event_next_id);
        self.local_event_next_id += 1;

        self.specify_event(name.as_str(), EventType::Local, events_to_bind, |_, evt_tag| {
            LocalEventCreator {
                local_event: LocalEvent::new(evt_tag),
            }
        })
    }

    pub(crate) fn specify_timer_event(
        &mut self,
        events_to_bind: &[Tag],
        cycle_duration: core::time::Duration,
    ) -> Result<EventCreator, CommonErrors> {
        let name = format!("timer_event_{}", self.timer_event_next_id);
        self.timer_event_next_id += 1;

        self.specify_event(name.as_str(), EventType::Timer, events_to_bind, |_, _| {
            TimerEventCreator { cycle: cycle_duration }
        })
    }

    fn specify_event<C, Ret>(
        &mut self,
        system_event: &str,
        t: EventType,
        events_to_bind: &[Tag],
        creator_builder: C,
    ) -> Result<EventCreator, CommonErrors>
    where
        C: FnOnce(&str, Tag) -> Ret,
        Ret: EventCreatorTrait + 'static,
    {
        let system_event_tag: Tag = system_event.into();

        if system_event_tag.is_in_collection(self.events.iter()) {
            warn!(
                "Event({:?}) with tag {:?} has been already bind, ignoring.",
                t, system_event_tag
            );
            return Err(CommonErrors::AlreadyDone);
        }

        trace!(
            "Binding event({:?}) {} to user events {:?}",
            t,
            system_event,
            events_to_bind
        );

        let creator: Rc<RefCell<dyn EventCreatorTrait>> =
            Rc::new(RefCell::new(creator_builder(system_event, system_event_tag)));

        self.events
            .push(DeploymentEventInfo {
                system_tag: system_event_tag,
                creator: Rc::clone(&creator),
            })
            .map_err(|_| CommonErrors::NoSpaceLeft)?;

        Ok(creator)
    }

    #[allow(dead_code)]
    pub(crate) fn get_event_creator(&self, system_event: &str) -> Option<EventCreator> {
        Some(Rc::clone(
            &Into::<Tag>::into(system_event)
                .find_in_collection(self.events.iter())?
                .creator,
        ))
    }
}

pub(crate) enum EventActionType {
    Sync,
    Trigger,
}

pub(crate) type EventCreator = Rc<RefCell<dyn EventCreatorTrait>>;

pub trait ShutdownNotifier {
    fn shutdown(&mut self) -> ActionResult;
}

struct ShutdownNotifierImpl<N: NotifierTrait> {
    notifier: N,
}

impl<N: NotifierTrait> ShutdownNotifier for ShutdownNotifierImpl<N> {
    fn shutdown(&mut self) -> ActionResult {
        self.notifier.notify_sync(0)
    }
}

pub(crate) trait EventCreatorTrait {
    fn create_trigger(&mut self, config: &DesignConfig) -> Option<Box<dyn ActionTrait>>;
    fn create_sync(&mut self, config: &DesignConfig) -> Option<Box<dyn ActionTrait>>;
    fn create_shutdown_notifier(&mut self) -> Option<Box<dyn ShutdownNotifier>>;
}

struct LocalEventCreator {
    local_event: LocalEvent,
}

impl EventCreatorTrait for LocalEventCreator {
    fn create_trigger(&mut self, config: &DesignConfig) -> Option<Box<dyn ActionTrait>> {
        let n = self.local_event.get_notifier();
        if n.is_none() {
            debug!("Failed to create Trigger Action, notifier is None. Did you tried to create two notifiers for the same event?");
        }

        Some(Trigger::new(n?, config.max_concurrent_action_executions) as Box<dyn ActionTrait>)
    }

    fn create_sync(&mut self, config: &DesignConfig) -> Option<Box<dyn ActionTrait>> {
        Some(Sync::new(
            self.local_event.get_listener()?,
            config.max_concurrent_action_executions,
        ) as Box<dyn ActionTrait>)
    }

    fn create_shutdown_notifier(&mut self) -> Option<Box<dyn ShutdownNotifier>> {
        let n = self.local_event.get_notifier();
        if n.is_none() {
            debug!("Failed to create Trigger Action, notifier is None. Did you tried to create two notifiers for the same event?");
        }

        Some(Box::new(ShutdownNotifierImpl { notifier: n? }))
    }
}

struct GlobalEventCreator<GlobalProvider: IpcProvider> {
    system_event_name: String,
    global_provider: Rc<RefCell<GlobalProvider>>,
}

impl<GlobalProvider: IpcProvider> EventCreatorTrait for GlobalEventCreator<GlobalProvider> {
    fn create_trigger(&mut self, config: &DesignConfig) -> Option<Box<dyn ActionTrait>> {
        Some(Trigger::new(
            self.global_provider
                .borrow_mut()
                .get_notifier(self.system_event_name.as_str())?,
            config.max_concurrent_action_executions,
        ) as Box<dyn ActionTrait>)
    }

    fn create_sync(&mut self, config: &DesignConfig) -> Option<Box<dyn ActionTrait>> {
        Some(Sync::new(
            self.global_provider
                .borrow_mut()
                .get_listener(self.system_event_name.as_str())?,
            config.max_concurrent_action_executions,
        ) as Box<dyn ActionTrait>)
    }

    fn create_shutdown_notifier(&mut self) -> Option<Box<dyn ShutdownNotifier>> {
        Some(Box::new(ShutdownNotifierImpl {
            notifier: self
                .global_provider
                .borrow_mut()
                .get_notifier(self.system_event_name.as_str())?,
        }))
    }
}

struct TimerEventCreator {
    cycle: Duration,
}

impl EventCreatorTrait for TimerEventCreator {
    fn create_trigger(&mut self, _config: &DesignConfig) -> Option<Box<dyn ActionTrait>> {
        panic!("Cannot create trigger for a event that is bound to a Timer Event type !")
    }

    fn create_sync(&mut self, config: &DesignConfig) -> Option<Box<dyn ActionTrait>> {
        Some(Sync::new(TimerEvent::new(self.cycle), config.max_concurrent_action_executions) as Box<dyn ActionTrait>)
    }

    fn create_shutdown_notifier(&mut self) -> Option<Box<dyn ShutdownNotifier>> {
        panic!("Cannot create trigger for a event that is bound to a Timer Event type for shutdown!")
    }
}

struct DeploymentEventInfo {
    system_tag: Tag,
    creator: EventCreator,
}

impl AsTagTrait for &DeploymentEventInfo {
    fn as_tag(&self) -> &Tag {
        &self.system_tag
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use super::*;
    use crate::testing::OrchTestingPoller;
    use kyron_foundation::prelude::CommonErrors;
    use kyron_testing::assert_poll_ready;

    #[test]
    fn new_provider() {
        let provider: EventsProvider = EventsProvider::new();
        assert_eq!(provider.events.len(), 0);
    }

    #[test]
    fn specify_event_duplicate() {
        let mut provider: EventsProvider = EventsProvider::new();

        provider
            .specify_event("100", EventType::Local, &["UserEvt".into()], |_, evt_tag| {
                LocalEventCreator {
                    local_event: LocalEvent::new(evt_tag),
                }
            })
            .unwrap();
        // Try to specify again with the same system tag
        let res = provider.specify_event("100", EventType::Local, &["UserEvt".into()], |_, evt_tag| {
            LocalEventCreator {
                local_event: LocalEvent::new(evt_tag),
            }
        });
        assert_eq!(res.err().unwrap(), CommonErrors::AlreadyDone);
    }

    #[test]
    fn creating_same_trigger_action_twice_causes_fail() {
        let config = DesignConfig::default();
        let mut provider: EventsProvider = EventsProvider::new();

        let res = provider.specify_event("100", EventType::Local, &["UserEvt".into()], |_, evt_tag| {
            LocalEventCreator {
                local_event: LocalEvent::new(evt_tag),
            }
        });
        assert!(res.is_ok());

        let creator = provider.get_event_creator("100").unwrap();

        let trigger_action = creator.borrow_mut().create_trigger(&config);

        assert!(trigger_action.is_some());
        assert!(creator.borrow_mut().create_trigger(&config).is_none());
    }

    #[test]
    fn creating_same_sync_action_n_times_works() {
        let config = DesignConfig::default();
        let mut provider: EventsProvider = EventsProvider::new();

        let res = provider.specify_event("100", EventType::Local, &["UserEvt".into()], |_, evt_tag| {
            LocalEventCreator {
                local_event: LocalEvent::new(evt_tag),
            }
        });
        assert!(res.is_ok());

        let creator = provider.get_event_creator("100").unwrap();

        let mut trigger_action = creator.borrow_mut().create_sync(&config);
        assert!(trigger_action.is_some());

        trigger_action = creator.borrow_mut().create_sync(&config);
        assert!(trigger_action.is_some());

        trigger_action = creator.borrow_mut().create_sync(&config);
        assert!(trigger_action.is_some());
    }

    #[test]
    fn sync_trigger_local_pair_works() {
        let config = DesignConfig::default();
        let mut provider: EventsProvider = EventsProvider::new();

        let res = provider.specify_event("100", EventType::Local, &["UserEvt".into()], |_, evt_tag| {
            LocalEventCreator {
                local_event: LocalEvent::new(evt_tag),
            }
        });
        assert!(res.is_ok());

        let mut trigger_action = provider
            .get_event_creator("100")
            .unwrap()
            .borrow_mut()
            .create_trigger(&config)
            .unwrap();
        let mut sync_action = provider
            .get_event_creator("100")
            .unwrap()
            .borrow_mut()
            .create_sync(&config)
            .unwrap();

        let trig_f = trigger_action.try_execute().unwrap();

        let sync_f = sync_action.try_execute().unwrap();

        let mut sync_poller = OrchTestingPoller::new(sync_f);
        let mut trigger_poller = OrchTestingPoller::new(trig_f);

        let mut ret = sync_poller.poll();
        assert!(ret.is_pending()); // Sync should be pending as no trigger has been called yet

        ret = trigger_poller.poll(); // Call trigger

        assert_poll_ready(ret, Ok(()));

        ret = sync_poller.poll(); // Now sync should be ready as trigger was called
        assert_poll_ready(ret, Ok(()));
    }

    #[test]
    fn sync_trigger_local_from_different_tag_does_not_unblock() {
        let config = DesignConfig::default();
        let mut provider: EventsProvider = EventsProvider::new();

        let mut res = provider.specify_event("100", EventType::Local, &["UserEvt".into()], |_, evt_tag| {
            LocalEventCreator {
                local_event: LocalEvent::new(evt_tag),
            }
        });
        assert!(res.is_ok());
        res = provider.specify_event("101", EventType::Local, &["UserEvt".into()], |_, evt_tag| {
            LocalEventCreator {
                local_event: LocalEvent::new(evt_tag),
            }
        });
        assert!(res.is_ok());

        let mut trigger_action = provider
            .get_event_creator("100")
            .unwrap()
            .borrow_mut()
            .create_trigger(&config)
            .unwrap();

        let mut sync_action = provider
            .get_event_creator("101")
            .unwrap()
            .borrow_mut()
            .create_sync(&config)
            .unwrap();

        let trig_f = trigger_action.try_execute().unwrap();

        let sync_f = sync_action.try_execute().unwrap();

        let mut sync_poller = OrchTestingPoller::new(sync_f);
        let mut trigger_poller = OrchTestingPoller::new(trig_f);

        let mut ret = sync_poller.poll();
        assert!(ret.is_pending()); // Sync should be pending as no trigger has been called yet

        ret = trigger_poller.poll(); // Call trigger

        assert_poll_ready(ret, Ok(()));

        ret = sync_poller.poll();
        assert!(ret.is_pending()); // Sync should be pending as  trigger was called for different event
    }
}
