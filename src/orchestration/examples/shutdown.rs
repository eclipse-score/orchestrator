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
use kyron::runtime::*;
use kyron_foundation::prelude::*;
use logging_tracing::{Level, LogAndTraceBuilder};
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
    prelude::Invoke,
};
use std::thread;

mod common;

fn example_component_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("ExampleDesign".into(), DesignConfig::default());
    let run_tag =
        design.register_invoke_async("PendingIndefinitely".into(), async || ::core::future::pending().await)?;
    let start_tag = design.register_invoke_fn("StartAction".into(), || {
        info!("Start action executed.");
        Ok(())
    })?;
    let stop_tag = design.register_invoke_fn("StopAction".into(), || {
        info!("Stop action executed.");
        Ok(())
    })?;

    design.add_program("ExampleDesignProgram", move |design, builder| {
        builder
            .with_run_action(Invoke::from_tag(&run_tag, design.config()))
            .with_start_action(Invoke::from_tag(&start_tag, design.config()))
            .with_stop_action(Invoke::from_tag(&stop_tag, design.config()), Duration::from_secs(5))
            .with_shutdown_event("ExampleShutdown".into());

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
    let (builder, _engine_id) = kyron::runtime::RuntimeBuilder::new()
        .with_engine(ExecutionEngineBuilder::new().task_queue_size(256).workers(2));
    let mut runtime = builder.build().unwrap();

    // Build Orchestration

    let mut orch = Orchestration::new()
        .add_design(example_component_design().expect("Failed to create design"))
        .design_done();

    // Deployment part - specify event details
    let mut deployment = orch.get_deployment_mut();
    deployment
        .bind_shutdown_event_as_local("ExampleShutdown".into())
        .expect("Failed to bind shutdown event");

    // Create program
    let mut program_manager = orch.into_program_manager().unwrap();
    let mut programs = program_manager.get_programs();
    let mut program = programs.pop().unwrap();
    let mut shutdown = program_manager
        .get_shutdown_notifier("ExampleShutdown".into())
        .expect("Failed to get shutdown notifier");

    // Put programs into runtime and run them
    runtime.spawn(async move {
        let _ = program.run().await;
        info!("Program finished running.");
    });

    info!("Runtime spawned");
    thread::sleep(Duration::from_secs(5));
    info!("Calling shutdown");
    let _ = shutdown.shutdown();

    info!("Exit.");
}
