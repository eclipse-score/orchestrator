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

use crate::actions::ifelse::{IfElse, IfElseCondition};
use crate::common::orch_tag::OrchestrationTag;
use crate::common::tag::Tag;
use crate::common::DesignConfig;
use crate::events::events_provider::EventActionType;
use crate::{
    actions::{
        action::ActionTrait,
        invoke::{Invoke, InvokeFunctionType, InvokeResult},
    },
    events::events_provider::EventCreator,
};
use iceoryx2_bb_container::flatmap::{FlatMap, FlatMapError};
use kyron::core::types::UniqueWorkerId;
use kyron_foundation::prelude::*;
use std::{
    boxed::Box,
    rc::Rc,
    sync::{Arc, Mutex},
};

use ::core::{cell::RefCell, fmt::Debug, future::Future};

pub(crate) struct ActionProvider {
    data: FlatMap<Tag, ActionData>,
}

impl ActionProvider {
    pub(crate) fn new(config: DesignConfig) -> Self {
        Self {
            data: FlatMap::new(config.db_params.registration_capacity),
        }
    }

    pub(crate) fn provide_invoke(&mut self, tag: Tag, config: &DesignConfig) -> Option<Box<dyn ActionTrait>> {
        self.data.get_ref(&tag).and_then(|data| match data {
            ActionData::Invoke(invoke_data) => Some((invoke_data.generator)(tag, invoke_data.worker_id, config)),
            _ => None,
        })
    }

    pub(crate) fn provide_event(
        &mut self,
        tag: Tag,
        t: EventActionType,
        config: &DesignConfig,
    ) -> Option<Box<dyn ActionTrait>> {
        self.data.get_ref(&tag).and_then(|data| match data {
            ActionData::Event(event_data) => match t {
                EventActionType::Trigger => event_data.creator()?.borrow_mut().create_trigger(config),
                EventActionType::Sync => event_data.creator()?.borrow_mut().create_sync(config),
            },
            _ => None,
        })
    }

    pub(crate) fn provide_if_else(
        &mut self,
        tag: Tag,
        true_branch: Box<dyn ActionTrait>,
        false_branch: Box<dyn ActionTrait>,
        config: &DesignConfig,
    ) -> Option<Box<dyn ActionTrait>> {
        self.data.get_ref(&tag).and_then(|data| match data {
            ActionData::IfElse(ifelse_data) => Some((ifelse_data.generator)(true_branch, false_branch, config)),
            _ => None,
        })
    }
}

impl Debug for ActionProvider {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "ActionProvider")
    }
}

pub struct ProgramDatabase {
    action_provider: Rc<RefCell<ActionProvider>>,
}

impl ProgramDatabase {
    /// Creates a new instance of `ProgramDatabase`.
    pub fn new(config: DesignConfig) -> Self {
        Self {
            action_provider: Rc::new(RefCell::new(ActionProvider::new(config))),
        }
    }

    /// Registers a function as an invoke action that can be created multiple times.
    pub fn register_invoke_fn(&self, tag: Tag, action: InvokeFunctionType) -> Result<OrchestrationTag, CommonErrors> {
        let mut ap = self.action_provider.borrow_mut();

        match ap.data.insert(
            tag,
            ActionData::Invoke(InvokeData {
                worker_id: None,
                generator: Rc::new(
                    move |tag: Tag, worker_id: Option<UniqueWorkerId>, config: &DesignConfig| {
                        Invoke::from_fn(tag, action, worker_id, config)
                    },
                ),
            }),
        ) {
            Ok(_) => Ok(OrchestrationTag::new(tag, Rc::clone(&self.action_provider))),
            Err(FlatMapError::IsFull) => Err(CommonErrors::NoSpaceLeft),
            Err(FlatMapError::KeyAlreadyExists) => Err(CommonErrors::AlreadyDone),
        }
    }

    /// Registers an async function as an invoke action that can be created multiple times.
    pub fn register_invoke_async<A, F>(&self, tag: Tag, action: A) -> Result<OrchestrationTag, CommonErrors>
    where
        A: Fn() -> F + 'static + Send + Clone,
        F: Future<Output = InvokeResult> + 'static + Send,
    {
        let mut ap = self.action_provider.borrow_mut();

        match ap.data.insert(
            tag,
            ActionData::Invoke(InvokeData {
                worker_id: None,
                generator: Rc::new(
                    move |tag: Tag, worker_id: Option<UniqueWorkerId>, config: &DesignConfig| {
                        Invoke::from_async(tag, action.clone(), worker_id, config)
                    },
                ),
            }),
        ) {
            Ok(_) => Ok(OrchestrationTag::new(tag, Rc::clone(&self.action_provider))),
            Err(FlatMapError::IsFull) => Err(CommonErrors::NoSpaceLeft),
            Err(FlatMapError::KeyAlreadyExists) => Err(CommonErrors::AlreadyDone),
        }
    }

    /// Registers a method on an object as an invoke action.
    pub fn register_invoke_method<T: 'static + Send>(
        &self,
        tag: Tag,
        object: Arc<Mutex<T>>,
        method: fn(&mut T) -> InvokeResult,
    ) -> Result<OrchestrationTag, CommonErrors> {
        let mut ap = self.action_provider.borrow_mut();

        match ap.data.insert(
            tag,
            ActionData::Invoke(InvokeData {
                worker_id: None,
                generator: Rc::new(
                    move |tag: Tag, worker_id: Option<UniqueWorkerId>, config: &DesignConfig| {
                        Invoke::from_method(tag, Arc::clone(&object), method, worker_id, config)
                    },
                ),
            }),
        ) {
            Ok(_) => Ok(OrchestrationTag::new(tag, Rc::clone(&self.action_provider))),
            Err(FlatMapError::IsFull) => Err(CommonErrors::NoSpaceLeft),
            Err(FlatMapError::KeyAlreadyExists) => Err(CommonErrors::AlreadyDone),
        }
    }

    /// Registers an async method on an object as an invoke action.
    pub fn register_invoke_method_async<T, M, F>(
        &self,
        tag: Tag,
        object: Arc<Mutex<T>>,
        method: M,
    ) -> Result<OrchestrationTag, CommonErrors>
    where
        T: 'static + Send,
        M: Fn(Arc<Mutex<T>>) -> F + 'static + Send + Clone,
        F: Future<Output = InvokeResult> + 'static + Send,
    {
        let mut ap = self.action_provider.borrow_mut();

        match ap.data.insert(
            tag,
            ActionData::Invoke(InvokeData {
                worker_id: None,
                generator: Rc::new(
                    move |tag: Tag, worker_id: Option<UniqueWorkerId>, config: &DesignConfig| {
                        Invoke::from_method_async(tag, Arc::clone(&object), method.clone(), worker_id, config)
                    },
                ),
            }),
        ) {
            Ok(_) => Ok(OrchestrationTag::new(tag, Rc::clone(&self.action_provider))),
            Err(FlatMapError::IsFull) => Err(CommonErrors::NoSpaceLeft),
            Err(FlatMapError::KeyAlreadyExists) => Err(CommonErrors::AlreadyDone),
        }
    }

    /// Registers an event for the Sync and Trigger actions.
    pub fn register_event(&self, tag: Tag) -> Result<OrchestrationTag, CommonErrors> {
        let mut ap = self.action_provider.borrow_mut();

        match ap.data.insert(tag, ActionData::Event(EventData { creator: None })) {
            Ok(_) => {
                trace!("Registered event with tag: {:?}", tag);
                Ok(OrchestrationTag::new(tag, Rc::clone(&self.action_provider)))
            },
            Err(FlatMapError::IsFull) => Err(CommonErrors::NoSpaceLeft),
            Err(FlatMapError::KeyAlreadyExists) => Err(CommonErrors::AlreadyDone),
        }
    }

    /// Registers an arc condition for an IfElse action.
    pub fn register_if_else_arc_condition<C>(
        &mut self,
        tag: Tag,
        condition: Arc<C>,
    ) -> Result<OrchestrationTag, CommonErrors>
    where
        C: IfElseCondition + Send + Sync + 'static,
    {
        let mut ap = self.action_provider.borrow_mut();

        match ap.data.insert(
            tag,
            ActionData::IfElse(IfElseData {
                generator: Rc::new(
                    move |true_branch: Box<dyn ActionTrait>,
                          false_branch: Box<dyn ActionTrait>,
                          config: &DesignConfig| {
                        IfElse::from_arc_condition(Arc::clone(&condition), true_branch, false_branch, config)
                    },
                ),
            }),
        ) {
            Ok(_) => Ok(OrchestrationTag::new(tag, Rc::clone(&self.action_provider))),
            Err(FlatMapError::IsFull) => Err(CommonErrors::NoSpaceLeft),
            Err(FlatMapError::KeyAlreadyExists) => Err(CommonErrors::AlreadyDone),
        }
    }

    /// Registers an arc mutex condition for an IfElse action.
    pub fn register_if_else_arc_mutex_condition<C>(
        &mut self,
        tag: Tag,
        condition: Arc<Mutex<C>>,
    ) -> Result<OrchestrationTag, CommonErrors>
    where
        C: IfElseCondition + Send + 'static,
    {
        let mut ap = self.action_provider.borrow_mut();

        match ap.data.insert(
            tag,
            ActionData::IfElse(IfElseData {
                generator: Rc::new(
                    move |true_branch: Box<dyn ActionTrait>,
                          false_branch: Box<dyn ActionTrait>,
                          config: &DesignConfig| {
                        IfElse::from_arc_mutex_condition(Arc::clone(&condition), true_branch, false_branch, config)
                    },
                ),
            }),
        ) {
            Ok(_) => Ok(OrchestrationTag::new(tag, Rc::clone(&self.action_provider))),
            Err(FlatMapError::IsFull) => Err(CommonErrors::NoSpaceLeft),
            Err(FlatMapError::KeyAlreadyExists) => Err(CommonErrors::AlreadyDone),
        }
    }

    /// Returns an `OrchestrationTag` for an action previously registered with the given tag.
    ///
    /// # Returns
    /// - `Ok(OrchestrationTag)` if the tag exists and is associated with an action.
    /// - `Err(CommonErrors::NotFound)` if the tag does not exist.
    ///
    pub fn get_orchestration_tag(&self, tag: Tag) -> Result<OrchestrationTag, CommonErrors> {
        if self.action_provider.borrow().data.contains(&tag) {
            Ok(OrchestrationTag::new(tag, Rc::clone(&self.action_provider)))
        } else {
            Err(CommonErrors::NotFound)
        }
    }

    /// Associates an invoke action with a tag with the given worker id.
    pub(crate) fn set_invoke_worker_id(&mut self, tag: Tag, worker_id: UniqueWorkerId) -> Result<(), CommonErrors> {
        let ap = &mut self.action_provider.borrow_mut();

        if let Some(data) = ap.data.get_mut_ref(&tag) {
            match data {
                ActionData::Invoke(invoke_data) => {
                    if invoke_data.worker_id.is_some() {
                        return Err(CommonErrors::AlreadyDone);
                    }

                    trace!("Setting worker id {:?} for invoke action with tag {:?}", worker_id, tag);
                    invoke_data.worker_id = Some(worker_id);

                    Ok(())
                },
                _ => Err(CommonErrors::NotFound),
            }
        } else {
            Err(CommonErrors::NotFound)
        }
    }

    pub(crate) fn set_creator_for_events(
        &self,
        creator: EventCreator,
        user_event_tags: &[Tag],
    ) -> Result<(), CommonErrors> {
        let mut ap = self.action_provider.borrow_mut();
        let mut ret = Ok(());

        for tag in user_event_tags {
            if let Some(data) = ap.data.get_mut_ref(tag) {
                match data {
                    ActionData::Event(event_data) => event_data.set_creator(Rc::clone(&creator), tag),
                    _ => ret = Err(CommonErrors::NotFound),
                }
            } else {
                ret = Err(CommonErrors::NotFound)
            }
        }

        ret
    }
}

impl Default for ProgramDatabase {
    fn default() -> Self {
        Self::new(DesignConfig::default())
    }
}

type InvokeGenerator = dyn Fn(Tag, Option<UniqueWorkerId>, &DesignConfig) -> Box<dyn ActionTrait>;
type IfElseGenerator = dyn Fn(Box<dyn ActionTrait>, Box<dyn ActionTrait>, &DesignConfig) -> Box<dyn ActionTrait>;

#[derive(Clone)]
struct InvokeData {
    worker_id: Option<UniqueWorkerId>,
    // Rc needed for Clone
    generator: Rc<InvokeGenerator>,
}

#[derive(Clone)]
struct EventData {
    creator: Option<EventCreator>,
}

impl EventData {
    pub fn creator(&self) -> Option<EventCreator> {
        self.creator.clone()
    }

    pub fn set_creator(&mut self, creator: EventCreator, tag: &Tag) {
        let prev = self.creator.replace(creator);
        if prev.is_some() {
            warn!(
                "Event with tag {:?} already has a binding, we replace it with new one provided.",
                tag
            );
        }
    }
}

#[derive(Clone)]
struct IfElseData {
    // Rc needed for Clone
    generator: Rc<IfElseGenerator>,
}

#[derive(Clone)]
enum ActionData {
    Invoke(InvokeData),
    Event(EventData),
    IfElse(IfElseData),
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use super::*;
    use crate::{
        actions::action::ActionExecError,
        events::events_provider::{EventCreatorTrait, ShutdownNotifier},
        testing::OrchTestingPoller,
    };
    use ::core::task::Poll;
    use kyron::testing;
    use kyron_testing_macros::ensure_clear_mock_runtime;

    #[test]
    fn test_register_invoke_fn() {
        let pd = ProgramDatabase::default();
        let config = DesignConfig::default();

        fn test1() -> InvokeResult {
            Err(0xcafe_u64.into())
        }

        fn test2() -> InvokeResult {
            Err(0xbeef_u64.into())
        }

        let tag = pd.register_invoke_fn("tag1".into(), test1).unwrap();
        assert!(pd.register_invoke_fn("tag1".into(), test1).is_err());
        assert!(pd.register_invoke_fn("tag2".into(), test2).is_ok());

        let mut invoke = Invoke::from_tag(&tag, &config);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());
        assert_eq!(
            poller.poll(),
            Poll::Ready(Err(ActionExecError::UserError(0xcafe_u64.into())))
        );

        let tag = pd.get_orchestration_tag("tag2".into()).unwrap();
        let mut invoke = Invoke::from_tag(&tag, &config);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());
        assert_eq!(
            poller.poll(),
            Poll::Ready(Err(ActionExecError::UserError(0xbeef_u64.into())))
        );
    }

    #[test]
    fn test_register_invoke_async() {
        let config = DesignConfig::default();
        let pd = ProgramDatabase::default();

        async fn test1() -> InvokeResult {
            Err(0xcafe_u64.into())
        }

        async fn test2() -> InvokeResult {
            Err(0xbeef_u64.into())
        }

        let tag = pd.register_invoke_async("tag1".into(), test1).unwrap();
        assert!(pd.register_invoke_async("tag1".into(), test1).is_err());
        assert!(pd.register_invoke_async("tag2".into(), test2).is_ok());

        let mut invoke = Invoke::from_tag(&tag, &config);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());
        assert_eq!(
            poller.poll(),
            Poll::Ready(Err(ActionExecError::UserError(0xcafe_u64.into())))
        );

        let tag = pd.get_orchestration_tag("tag2".into()).unwrap();
        let mut invoke = Invoke::from_tag(&tag, &config);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());
        assert_eq!(
            poller.poll(),
            Poll::Ready(Err(ActionExecError::UserError(0xbeef_u64.into())))
        );
    }

    #[test]
    fn test_register_invoke_method() {
        let config = DesignConfig::default();
        let pd = ProgramDatabase::default();

        struct Test1 {}

        impl Test1 {
            fn test1(&mut self) -> InvokeResult {
                Err(0xcafe_u64.into())
            }
        }

        struct Test2 {}

        impl Test2 {
            fn test2(&mut self) -> InvokeResult {
                Err(0xbeef_u64.into())
            }
        }

        let obj1 = Arc::new(Mutex::new(Test1 {}));
        let obj2 = Arc::new(Mutex::new(Test2 {}));

        let tag = pd
            .register_invoke_method("tag1".into(), Arc::clone(&obj1), Test1::test1)
            .unwrap();
        assert!(pd
            .register_invoke_method("tag1".into(), Arc::clone(&obj1), Test1::test1)
            .is_err());
        assert!(pd
            .register_invoke_method("tag2".into(), Arc::clone(&obj2), Test2::test2)
            .is_ok());

        let mut invoke = Invoke::from_tag(&tag, &config);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());
        assert_eq!(
            poller.poll(),
            Poll::Ready(Err(ActionExecError::UserError(0xcafe_u64.into())))
        );

        let tag = pd.get_orchestration_tag("tag2".into()).unwrap();
        let mut invoke = Invoke::from_tag(&tag, &config);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());
        assert_eq!(
            poller.poll(),
            Poll::Ready(Err(ActionExecError::UserError(0xbeef_u64.into())))
        );
    }

    #[test]
    fn test_register_invoke_method_async() {
        let config = DesignConfig::default();
        let pd = ProgramDatabase::default();

        struct Test1 {}

        async fn test1(object: Arc<Mutex<Test1>>) -> InvokeResult {
            let _guard = object.lock().unwrap();
            Err(0xcafe_u64.into())
        }

        struct Test2 {}

        async fn test2(object: Arc<Mutex<Test2>>) -> InvokeResult {
            let _guard = object.lock().unwrap();
            Err(0xbeef_u64.into())
        }

        let obj1 = Arc::new(Mutex::new(Test1 {}));
        let obj2 = Arc::new(Mutex::new(Test2 {}));

        let tag = pd
            .register_invoke_method_async("tag1".into(), Arc::clone(&obj1), test1)
            .unwrap();
        assert!(pd
            .register_invoke_method_async("tag1".into(), Arc::clone(&obj1), test1)
            .is_err());
        assert!(pd
            .register_invoke_method_async("tag2".into(), Arc::clone(&obj2), test2)
            .is_ok());

        let mut invoke = Invoke::from_tag(&tag, &config);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());
        assert_eq!(
            poller.poll(),
            Poll::Ready(Err(ActionExecError::UserError(0xcafe_u64.into())))
        );

        let tag = pd.get_orchestration_tag("tag2".into()).unwrap();
        let mut invoke = Invoke::from_tag(&tag, &config);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());
        assert_eq!(
            poller.poll(),
            Poll::Ready(Err(ActionExecError::UserError(0xbeef_u64.into())))
        );
    }

    #[test]
    #[ensure_clear_mock_runtime]
    fn test_invoke_fn_with_worker_id() {
        let config = DesignConfig::default();
        let mut pd = ProgramDatabase::default();

        fn test1() -> InvokeResult {
            Err(0xcafe_u64.into())
        }

        let tag = pd.register_invoke_fn("tag1".into(), test1).unwrap();
        assert_eq!(pd.set_invoke_worker_id("tag1".into(), "worker_id".into()), Ok(()));
        let mut invoke = Invoke::from_tag(&tag, &config);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());

        // Wait for invoke to schedule the action.
        let _ = poller.poll();
        // Run the action.
        assert!(testing::mock::runtime::remaining_tasks() > 0);
        testing::mock::runtime::step();
        assert_eq!(testing::mock::runtime::remaining_tasks(), 0);
        // Check the result.
        assert_eq!(
            poller.poll(),
            Poll::Ready(Err(ActionExecError::UserError(0xcafe_u64.into())))
        );
    }

    #[test]
    #[ensure_clear_mock_runtime]
    fn test_invoke_async_with_worker_id() {
        let config = DesignConfig::default();
        let mut pd = ProgramDatabase::default();

        async fn test1() -> InvokeResult {
            Err(0xcafe_u64.into())
        }

        let tag = pd.register_invoke_async("tag1".into(), test1).unwrap();
        assert_eq!(pd.set_invoke_worker_id("tag1".into(), "worker_id".into()), Ok(()));
        let mut invoke = Invoke::from_tag(&tag, &config);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());

        // Wait for invoke to schedule the action.
        let _ = poller.poll();
        // Run the action.
        assert!(testing::mock::runtime::remaining_tasks() > 0);
        testing::mock::runtime::step();
        assert_eq!(testing::mock::runtime::remaining_tasks(), 0);
        // Check the result.
        assert_eq!(
            poller.poll(),
            Poll::Ready(Err(ActionExecError::UserError(0xcafe_u64.into())))
        );
    }

    #[test]
    #[ensure_clear_mock_runtime]
    fn test_invoke_method_with_worker_id() {
        let config = DesignConfig::default();
        let mut pd = ProgramDatabase::default();

        struct Test1 {}

        impl Test1 {
            fn test1(&mut self) -> InvokeResult {
                Err(0xcafe_u64.into())
            }
        }

        let tag = pd
            .register_invoke_method("tag1".into(), Arc::new(Mutex::new(Test1 {})), Test1::test1)
            .unwrap();
        assert_eq!(pd.set_invoke_worker_id("tag1".into(), "worker_id".into()), Ok(()));
        let mut invoke = Invoke::from_tag(&tag, &config);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());

        // Wait for invoke to schedule the action.
        let _ = poller.poll();
        // Run the action.
        assert!(testing::mock::runtime::remaining_tasks() > 0);
        testing::mock::runtime::step();
        assert_eq!(testing::mock::runtime::remaining_tasks(), 0);
        // Check the result.
        assert_eq!(
            poller.poll(),
            Poll::Ready(Err(ActionExecError::UserError(0xcafe_u64.into())))
        );
    }

    #[test]
    #[ensure_clear_mock_runtime]
    fn test_invoke_method_async_with_worker_id() {
        let config = DesignConfig::default();
        let mut pd = ProgramDatabase::default();

        struct Test1 {}

        async fn test1(object: Arc<Mutex<Test1>>) -> InvokeResult {
            let _guard = object.lock().unwrap();
            Err(0xcafe_u64.into())
        }

        let tag = pd
            .register_invoke_method_async("tag1".into(), Arc::new(Mutex::new(Test1 {})), test1)
            .unwrap();
        assert_eq!(pd.set_invoke_worker_id("tag1".into(), "worker_id".into()), Ok(()));
        let mut invoke = Invoke::from_tag(&tag, &config);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());

        // Wait for invoke to schedule the action.
        let _ = poller.poll();
        // Run the action.
        assert!(testing::mock::runtime::remaining_tasks() > 0);
        testing::mock::runtime::step();
        assert_eq!(testing::mock::runtime::remaining_tasks(), 0);
        // Check the result.
        assert_eq!(
            poller.poll(),
            Poll::Ready(Err(ActionExecError::UserError(0xcafe_u64.into())))
        );
    }

    fn make_tag(val: u32) -> Tag {
        val.to_string().as_str().into()
    }

    #[test]
    fn register_event_success() {
        let pd = ProgramDatabase::default();
        let tag = make_tag(1);

        let orch_tag = pd.register_event(tag);
        assert!(orch_tag.is_ok());
        let found = pd.get_orchestration_tag(tag);
        assert_eq!(tag, *(found.unwrap().tag()));
    }

    #[test]
    fn register_same_event_twice() {
        let pd = ProgramDatabase::default();
        let tag = make_tag(1);

        let mut orch_tag = pd.register_event(tag);
        assert!(orch_tag.is_ok());

        orch_tag = pd.register_event(tag);
        assert!(orch_tag.is_err());
    }

    #[test]
    fn register_event_no_space_left() {
        let config = DesignConfig::default();
        let pd = ProgramDatabase::new(config);

        // Fill up the slotmap to its capacity
        for i in 0..config.db_params.registration_capacity {
            let tag = make_tag(i as u32);
            let res = pd.register_event(tag);
            assert!(res.is_ok());
        }
        // Next insert should fail
        let tag = make_tag(9999);
        let res = pd.register_event(tag);
        assert_eq!(res.unwrap_err(), CommonErrors::NoSpaceLeft);
    }

    #[test]
    fn specify_event_local_success() {
        let pd = ProgramDatabase::default();

        let tag1 = make_tag(1);
        let tag2 = make_tag(2);
        pd.register_event(tag1).unwrap();
        pd.register_event(tag2).unwrap();

        struct TestEventCreator {}

        impl EventCreatorTrait for TestEventCreator {
            fn create_trigger(&mut self, _: &DesignConfig) -> Option<Box<dyn ActionTrait>> {
                todo!()
            }

            fn create_sync(&mut self, _: &DesignConfig) -> Option<Box<dyn ActionTrait>> {
                todo!()
            }

            fn create_shutdown_notifier(&mut self) -> Option<Box<dyn ShutdownNotifier>> {
                todo!()
            }
        }

        let creator: EventCreator = Rc::new(RefCell::new(TestEventCreator {}));

        let res = pd.set_creator_for_events(creator, &[tag1, tag2]);
        assert!(res.is_ok());

        // Both design events should have a creator now
        let orch1 = pd.get_orchestration_tag(tag1).unwrap();
        let orch2 = pd.get_orchestration_tag(tag2).unwrap();

        fn get_event_creator(ap: Rc<RefCell<ActionProvider>>, tag: &Tag) -> Option<EventCreator> {
            ap.borrow().data.get_ref(tag).and_then(|data| match data {
                ActionData::Event(event_data) => event_data.creator().clone(),
                _ => None,
            })
        }

        let c1 = get_event_creator(Rc::clone(&pd.action_provider), orch1.tag());
        let c2 = get_event_creator(Rc::clone(&pd.action_provider), orch2.tag());

        assert!(Rc::ptr_eq(&c1.unwrap(), &c2.unwrap()));
    }
}
