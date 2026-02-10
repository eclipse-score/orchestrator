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

use kyron::spawn;
use kyron_foundation::prelude::{vector_extension::VectorExtension, *};
use logging_tracing::{Level, LogAndTraceBuilder};
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
    prelude::*,
    program::Program,
};
mod common;
use common::register_all_common_into_design;

pub fn dedicated_worker_func() -> InvokeResult {
    info!("Start of 'dedicated_worker_func' function.");

    info!("End of 'dedicated_worker_func' function.");
    Ok(())
}

fn example_component_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("ExampleDesign".into(), DesignConfig::default());

    register_all_common_into_design(&mut design)?; // Register our common functions, events, etc
    design.register_invoke_fn("dedicated_worker_func".into(), dedicated_worker_func)?;

    design.register_event("cyclic_evt".into())?; // Register a timer event

    // Create a program with some actions
    design.add_program("ExampleDesignProgram", move |design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(SyncBuilder::from_design("cyclic_evt", design_instance))
                .with_step(Invoke::from_design("dedicated_worker_func", design_instance))
                .with_step(Invoke::from_design("test2_sync_func", design_instance))
                .with_step(
                    ConcurrencyBuilder::new()
                        .with_branch(Invoke::from_design("test3_sync_func", design_instance))
                        .with_branch(Invoke::from_design("test4_sync_func", design_instance))
                        .build(design_instance),
                )
                .with_step(Invoke::from_design("test4_async_func", design_instance))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

// Programs are created in a separate function to avoid `Send` issue
fn create_orch_programs() -> Vec<Program> {
    // Build Orchestration
    let mut orch = Orchestration::new()
        .add_design(example_component_design().expect("Failed to create design"))
        .design_done();

    // Deployment part - specify event details
    let mut deployment = orch.get_deployment_mut();

    // Bind a invoke action to a dedicated worker
    deployment
        .bind_invoke_to_worker("dedicated_worker_func".into(), "dedicated_worker1".into())
        .expect("Failed to bind invoke action to worker");

    deployment
        .bind_events_as_timer(&["cyclic_evt".into()], Duration::from_secs(1))
        .expect("Failed to bind cycle event to timer");

    // Create programs
    let mut program_manager = orch.into_program_manager().unwrap();

    // Return the created programs
    program_manager.get_programs()
}

#[kyron::main(
    task_queue_size = 64,
    worker_threads = 2,
    dedicated_workers = [
        { id = "dedicated_worker1" }
    ],
)]
async fn main() {
    // Setup any logging framework you want to use.
    let _logger = LogAndTraceBuilder::new()
        .global_log_level(Level::INFO)
        //.enable_tracing(TraceScope::AppScope)
        .enable_logging(true)
        .build()
        .expect("Failed to build tracing library");

    // Get programs to run
    let mut programs = create_orch_programs();

    // Spawn all programs
    let mut handles = Vec::new_in_global(programs.len());
    while let Some(mut program) = programs.pop() {
        let handle = spawn(async move { program.run_n(3).await });
        let _ = handles.push(handle);
    }
    for handle in handles.iter_mut() {
        let _ = handle.await;
    }
    info!("Programs finished running.");
}
