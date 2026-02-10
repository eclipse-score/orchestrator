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

// Additional remarks:
//
// - Actions shall follow API aka build pattern (more or less like in Nico code) for construction
// - First we need tree actions: Sequence, Concurrent and Invoke
// - Invoke shall be able to take from user - a function, an async function and object + method for a moment.
//

use crate::{
    api::ShutdownEvent,
    common::{tag::Tag, DesignConfig},
    core::metering::{MeterTrait, NoneMeter},
    prelude::{ActionExecError, ActionResult, ActionTrait},
};
use ::core::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use kyron::{time::clock::Clock, JoinHandle};
use kyron_foundation::prelude::*;
use kyron_foundation::{containers::growable_vec::GrowableVec, prelude::CommonErrors};

#[cfg(not(any(test, feature = "runtime-api-mock")))]
use kyron::safety::spawn_from_reusable;
#[cfg(any(test, feature = "runtime-api-mock"))]
use kyron::testing::mock::safety::spawn_from_reusable;

///
/// Whole description to Task Chain is delivered via this instance. It shall hold all actions that build as Task Chain
///
pub struct Program {
    pub(crate) name: String,
    run_action: Box<dyn ActionTrait>,
    start_action: Option<Box<dyn ActionTrait>>,
    stop_action: Option<Box<dyn ActionTrait>>,
    #[allow(dead_code)]
    stop_timeout: Duration,
    shutdown_sync: Option<Box<dyn ActionTrait>>,
}

impl Debug for Program {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        writeln!(f, "Program - {}", self.name)?;
        writeln!(f, "Body:")?;
        self.run_action.as_ref().dbg_fmt(1, f)
    }
}

pub struct ProgramBuilder {
    name: String,
    run_action: Option<Box<dyn ActionTrait>>,
    start_action: Option<Box<dyn ActionTrait>>,
    stop_action: Option<Box<dyn ActionTrait>>,
    stop_timeout: Duration,
    shutdown_event_tag: Option<Tag>,
}

impl ProgramBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            run_action: None,
            start_action: None,
            stop_action: None,
            stop_timeout: Default::default(),
            shutdown_event_tag: None,
        }
    }

    pub fn with_run_action(&mut self, action: Box<dyn ActionTrait>) -> &mut Self {
        self.run_action = Some(action);
        self
    }

    pub fn with_start_action(&mut self, action: Box<dyn ActionTrait>) -> &mut Self {
        self.start_action = Some(action);
        self
    }

    pub fn with_stop_action(&mut self, action: Box<dyn ActionTrait>, timeout: Duration) -> &mut Self {
        self.stop_action = Some(action);
        // TODO(#151): The timeout is currently unused.
        self.stop_timeout = timeout;
        self
    }

    pub fn with_shutdown_event(&mut self, name: Tag) -> &mut Self {
        self.shutdown_event_tag = Some(name);
        self
    }

    pub(crate) fn build(
        self,
        shutdown_events: &GrowableVec<ShutdownEvent>,
        config: &DesignConfig,
    ) -> Result<Program, CommonErrors> {
        if self.run_action.is_none() {
            trace!("Missing run action");
            return Err(CommonErrors::NoData);
        }

        let mut shutdown_sync = None;

        if let Some(tag) = self.shutdown_event_tag {
            if let Some(shutdown_event) = tag.find_in_collection(shutdown_events.iter()) {
                shutdown_sync = shutdown_event.creator().borrow_mut().create_sync(config);
            } else {
                trace!("Shutdown event {} not found", tag.tracing_str());
                return Err(CommonErrors::NotFound);
            }
        }

        Ok(Program {
            name: self.name,
            run_action: self.run_action.unwrap(),
            start_action: self.start_action,
            stop_action: self.stop_action,
            stop_timeout: self.stop_timeout,
            shutdown_sync,
        })
    }
}

impl Program {
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Execute the run action in an infinite loop.
    pub async fn run(&mut self) -> ActionResult {
        self.internal_run::<NoneMeter>(None, None).await
    }

    /// Execute the run action a given number of times.
    pub async fn run_n(&mut self, n: usize) -> ActionResult {
        self.internal_run::<NoneMeter>(Some(n), None).await
    }

    /// Execute the run action in an infinite loop using `T` to measure the time taken for each iteration.
    pub async fn run_metered<T: MeterTrait>(&mut self) -> ActionResult {
        self.internal_run::<T>(None, None).await
    }

    /// Execute the run action a given number of times using `T` to measure the time taken for each iteration.
    pub async fn run_n_metered<T: MeterTrait>(&mut self, n: usize) -> ActionResult {
        self.internal_run::<T>(Some(n), None).await
    }

    /// Execute the run action a given number of times with a specified cycle duration.
    /// `cycle` is the time the whole iteration should take (execution + wait time).
    /// ATTENTION: Currently this is `dev` feature that does BLOCKING sleep
    pub async fn run_n_cycle(&mut self, n: usize, cycle: Duration) -> ActionResult {
        self.internal_run::<NoneMeter>(Some(n), Some(cycle)).await
    }

    /// Execute the run action with a specified cycle duration. `cycle` is the time the whole iteration should take (execution + wait time).
    /// ATTENTION: Currently this is `dev` feature that does BLOCKING sleep
    pub async fn run_cycle(&mut self, cycle: Duration) -> ActionResult {
        self.internal_run::<NoneMeter>(None, Some(cycle)).await
    }

    /// Execute the run action a given number of times with a specified cycle duration using `T` to measure the time taken for each iteration.
    /// `cycle` is the time the whole iteration should take (execution + wait time).
    /// ATTENTION: Currently this is `dev` feature that does BLOCKING sleep
    pub async fn run_n_cycle_metered<T: MeterTrait>(&mut self, n: usize, cycle: Duration) -> ActionResult {
        self.internal_run::<T>(Some(n), Some(cycle)).await
    }

    /// Execute the run action with a specified cycle duration using `T` to measure the time taken for each iteration.
    /// `cycle` is the time the whole iteration should take (execution + wait time).
    /// ATTENTION: Currently this is `dev` feature that does BLOCKING sleep
    pub async fn run_cycle_metered<T: MeterTrait>(&mut self, cycle: Duration) -> ActionResult {
        self.internal_run::<T>(None, Some(cycle)).await
    }

    async fn internal_run<T: MeterTrait>(&mut self, n: Option<usize>, cycle: Option<Duration>) -> ActionResult {
        let iteration_count: usize = n.unwrap_or_default();
        let mut iteration = 0_usize;
        let mut shutdown_handle = self.create_shutdown_handle()?;

        // Stop execution if the start action is present and results in an error.
        self.run_start_action().await?;

        let mut meter: T = T::new(self.name.as_str().into());

        while n.is_none() || iteration < iteration_count {
            let start_time = Clock::now();

            let run_future = self.run_action.as_mut().try_execute();
            if run_future.is_err() {
                trace!("Failed to execute run action");
                return Err(ActionExecError::Internal);
            }

            let mut run_handle = spawn_from_reusable(run_future.unwrap());
            let join_either = JoinEither {
                run_handle: &mut run_handle,
                shutdown_handle: &mut shutdown_handle,
            };

            match join_either.await {
                Ok(result) => match result.0 {
                    JoinedHandle::Run => result.1?,
                    JoinedHandle::Shutdown => break, // Not checking for ActionExecError on a Sync action.
                },
                Err(_) => {
                    trace!("Failed to execute run action or shutdown sync");
                    return Err(ActionExecError::Internal);
                },
            };

            let iteration_duration = start_time.elapsed();

            meter.meter(&iteration_duration, ("iteration", iteration));

            if let Some(cycle_duration) = cycle {
                if iteration_duration < cycle_duration {
                    std::thread::sleep(cycle_duration - iteration_duration);
                }
            }

            iteration += 1;
        }

        self.run_stop_action().await
    }

    async fn run_start_action(&mut self) -> ActionResult {
        if let Some(ref mut start_action) = self.start_action.take() {
            match start_action.try_execute() {
                Ok(future) => match spawn_from_reusable(future).await {
                    Ok(result) => result,
                    Err(_) => Err(ActionExecError::Internal),
                },
                Err(_) => Err(ActionExecError::Internal),
            }
        } else {
            Ok(())
        }
    }

    async fn run_stop_action(&mut self) -> ActionResult {
        if let Some(ref mut stop_action) = self.stop_action.take() {
            match stop_action.try_execute() {
                Ok(future) => match spawn_from_reusable(future).await {
                    Ok(result) => result,
                    Err(_) => Err(ActionExecError::Internal),
                },
                Err(_) => Err(ActionExecError::Internal),
            }
        } else {
            Ok(())
        }
    }

    fn create_shutdown_handle(&mut self) -> Result<Option<JoinHandle<ActionResult>>, ActionExecError> {
        if let Some(ref mut shutdown_sync) = self.shutdown_sync {
            match shutdown_sync.try_execute() {
                Ok(future) => Ok(Some(spawn_from_reusable(future))),
                Err(_) => Err(ActionExecError::Internal),
            }
        } else {
            Ok(None)
        }
    }
}

enum JoinedHandle {
    Run,
    Shutdown,
}

struct JoinEither<'a> {
    run_handle: &'a mut JoinHandle<ActionResult>,
    shutdown_handle: &'a mut Option<JoinHandle<ActionResult>>,
}

impl Future for JoinEither<'_> {
    type Output = Result<(JoinedHandle, ActionResult), CommonErrors>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut pin_a = Pin::new(&mut self.run_handle);
        match pin_a.as_mut().poll(cx) {
            Poll::Ready(result) => match result {
                Ok(result) => return Poll::Ready(Ok((JoinedHandle::Run, result))),
                Err(_) => return Poll::Ready(Err(CommonErrors::GenericError)),
            },
            Poll::Pending => (),
        }

        if let Some(ref mut shutdown_handle) = self.shutdown_handle {
            let mut pin_b = Pin::new(shutdown_handle);
            match pin_b.as_mut().poll(cx) {
                Poll::Ready(result) => match result {
                    Ok(result) => return Poll::Ready(Ok((JoinedHandle::Shutdown, result))),
                    Err(_) => return Poll::Ready(Err(CommonErrors::GenericError)),
                },
                Poll::Pending => (),
            }
        }

        Poll::Pending
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use super::*;
    use crate::{
        api::design::Design,
        common::DesignConfig,
        prelude::{Invoke, InvokeResult},
    };
    use core::time::Duration;
    use kyron::testing;
    use kyron_testing_macros::ensure_clear_mock_runtime;
    use std::sync::{Arc, Mutex};

    #[test]
    #[ensure_clear_mock_runtime]
    fn test_start_and_stop_action() {
        let design = Design::new("ExampleDesign".into(), DesignConfig::default());

        struct Flags {
            start_called: bool,
            run_called: bool,
            stop_called: bool,
        }

        impl Flags {
            fn new() -> Self {
                Self {
                    start_called: false,
                    run_called: false,
                    stop_called: false,
                }
            }

            fn start(&mut self) -> InvokeResult {
                self.start_called = true;
                Ok(())
            }

            fn run(&mut self) -> InvokeResult {
                self.run_called = true;
                Ok(())
            }

            fn stop(&mut self) -> InvokeResult {
                self.stop_called = true;
                Ok(())
            }
        }

        let flags = Arc::new(Mutex::new(Flags::new()));
        let start_tag = design
            .register_invoke_method("StartAction".into(), Arc::clone(&flags), Flags::start)
            .unwrap();
        let run_tag = design
            .register_invoke_method("RunAction".into(), Arc::clone(&flags), Flags::run)
            .unwrap();
        let stop_tag = design
            .register_invoke_method("StopAction".into(), Arc::clone(&flags), Flags::stop)
            .unwrap();

        let mut builder = ProgramBuilder::new("TestBuilder");
        builder
            .with_start_action(Invoke::from_tag(&start_tag, design.config()))
            .with_run_action(Invoke::from_tag(&run_tag, design.config()))
            .with_stop_action(Invoke::from_tag(&stop_tag, design.config()), Duration::from_secs(10));

        let mut program = builder.build(&GrowableVec::default(), design.config()).unwrap();
        testing::mock::spawn(async move {
            program.run_n(1).await.unwrap();
        });

        for _ in 0..10 {
            testing::mock::runtime::step();
        }

        let flags = flags.lock().unwrap();
        assert!(flags.start_called);
        assert!(flags.run_called);
        assert!(flags.stop_called);
    }
}
