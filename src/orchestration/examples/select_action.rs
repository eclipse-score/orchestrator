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

use core::time::Duration;

use kyron::prelude::*;
use kyron_foundation::prelude::*;
use logging_tracing::{Level, LogAndTraceBuilder};
use orchestration::{
    actions::select::SelectBuilder,
    api::{design::Design, Orchestration},
    common::DesignConfig,
    prelude::*,
};

mod common;
use common::register_all_common_into_design;

use kyron::futures::sleep;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct SampleReceive {
    recv_duration_ms: u64,
}

impl SampleReceive {
    fn new() -> Self {
        Self { recv_duration_ms: 0 }
    }

    fn increase_recv_duration(&mut self, increment_ms: u64) {
        self.recv_duration_ms += increment_ms;
    }

    async fn receive_data(&mut self) -> InvokeResult {
        // Simulate receiving data asynchronously
        info!("Start receiving data..........");
        sleep::sleep(::core::time::Duration::from_millis(self.recv_duration_ms)).await;
        info!("Received data after {} ms", self.recv_duration_ms);
        Ok(())
    }

    async fn timeout_500msec(&mut self) -> InvokeResult {
        sleep::sleep(::core::time::Duration::from_millis(500)).await;
        error!("Receive data timed out after 500 ms");
        Ok(())
    }
}

fn example_component_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("ExampleDesign".into(), DesignConfig::default());

    register_all_common_into_design(&mut design)?; // Register our common functions, events, etc

    // Create instance for SampleReceive
    let sample_recv = Arc::new(Mutex::new(SampleReceive::new()));
    design.register_invoke_method_async("receive_data".into(), sample_recv.clone(), |sr| {
        let mut guard = sr.lock().unwrap();
        guard.increase_recv_duration(200); // Increase duration each time it's called
        let mut sample = guard.clone();
        Box::pin(async move { sample.receive_data().await })
    })?;

    design.register_invoke_method_async("timeout_500msec".into(), sample_recv.clone(), |sr| {
        let guard = sr.lock().unwrap();
        let mut sample = guard.clone();
        Box::pin(async move { sample.timeout_500msec().await })
    })?;

    design.register_event("cyclic_evt".into())?; // Register a timer event

    // Create a program with some actions
    design.add_program("ExampleDesignProgram", move |design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(SyncBuilder::from_design("cyclic_evt", design_instance))
                .with_step(Invoke::from_design("test1_sync_func", design_instance))
                .with_step(Invoke::from_design("test2_sync_func", design_instance))
                .with_step(
                    SelectBuilder::new()
                        .with_case(Invoke::from_design("receive_data", design_instance))
                        .with_case(Invoke::from_design("timeout_500msec", design_instance))
                        .build(design_instance),
                )
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

fn main() {
    // Setup any logging framework you want to use.
    let _logger = LogAndTraceBuilder::new()
        .global_log_level(Level::INFO)
        //.enable_tracing(TraceScope::AppScope)
        .enable_logging(true)
        .build()
        .expect("Failed to build tracing library");

    // Create runtime
    let (builder, _engine_id) = kyron::runtime::RuntimeBuilder::new().with_engine(
        ExecutionEngineBuilder::new()
            .task_queue_size(256)
            .workers(2)
            .with_dedicated_worker("dedicated_worker1".into(), ThreadParameters::default()),
    );

    let mut runtime = builder.build().unwrap();

    // Build Orchestration

    let mut orch = Orchestration::new()
        .add_design(example_component_design().expect("Failed to create design"))
        .design_done();

    // Deployment part - specify event details
    let mut deployment = orch.get_deployment_mut();

    // Bind a invoke action to a dedicated worker
    deployment
        .bind_invoke_to_worker("test1_sync_func".into(), "dedicated_worker1".into())
        .expect("Failed to bind invoke action to worker");

    deployment
        .bind_events_as_timer(&["cyclic_evt".into()], Duration::from_secs(1))
        .expect("Failed to bind cycle event to timer");

    // Create programs
    let mut program_manager = orch.into_program_manager().unwrap();
    let mut programs = program_manager.get_programs();

    // Put programs into runtime and run them
    runtime.block_on(async move {
        let _ = programs.pop().unwrap().run_n(2).await;
        info!("Program finished running.");
    });

    info!("Exit.");
}
