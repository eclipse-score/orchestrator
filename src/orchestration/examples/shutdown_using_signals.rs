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

use ::core::future;
use kyron::runtime::*;
use kyron_foundation::prelude::*;
use logging_tracing::{Level, LogAndTraceBuilder};
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
    prelude::Invoke,
};

#[path = "common/signal_handler.rs"]
mod signal_handler;
use signal_handler::SignalHandler;

fn example_component_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("ExampleDesign".into(), DesignConfig::default());
    let t1 = design.register_invoke_async("PendingIndefinitely".into(), async || future::pending().await)?;

    design.add_program("ExampleDesignProgram", move |design, builder| {
        builder
            .with_run_action(Invoke::from_tag(&t1, design.config()))
            .with_shutdown_event("ShutdownEvent".into());

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
        .bind_shutdown_event_as_local("ShutdownEvent".into())
        .expect("Failed to bind shutdown event");

    // Create program
    let mut program_manager = orch.into_program_manager().unwrap();
    let mut programs = program_manager.get_programs();
    let mut program = programs.pop().unwrap();
    // Get shutdown notifier to shutdown the program when shutdown is requested
    let mut shutdown_notifier = program_manager
        .get_shutdown_notifier("ShutdownEvent".into())
        .expect("Failed to get shutdown notifier");

    // Register signal handlers for SIGINT and SIGTERM
    unsafe { SignalHandler::get_instance().register_signal_handlers() };

    // Put programs into runtime and run them
    runtime.spawn(async move {
        let _ = program.run().await;
        info!("Program terminated.");
    });
    info!("Runtime spawned. Running the program...");
    info!("Press Ctrl+C or send SIGTERM to terminate the program.");

    // Wait for shutdown signal
    let received_signal = SignalHandler::get_instance().wait_until_signal_received();
    info!("Received signal: {}", received_signal);

    // Shutdown the program
    let _ = shutdown_notifier.shutdown();

    info!("Exit.");
}
