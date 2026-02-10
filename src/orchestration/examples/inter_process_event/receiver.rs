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

use kyron::runtime::*;
use kyron_foundation::prelude::*;
use logging_tracing::{Level, LogAndTraceBuilder};
use orchestration::{
    api::{design::Design, Orchestration},
    common::{tag::Tag, DesignConfig},
    prelude::*,
};

pub fn apply_brake() -> InvokeResult {
    info!("Applying brake...");
    Ok(())
}

fn braking_component_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("BrakingDesign".into(), DesignConfig::default());

    design.register_invoke_fn("apply_brake".into(), apply_brake)?;
    design.register_event(Tag::from_str_static("BrakeEvent"))?;

    // Create a program with some actions
    design.add_program("BrakingProgram", move |design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(SyncBuilder::from_design("BrakeEvent", design_instance))
                .with_step(Invoke::from_design("apply_brake", design_instance))
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
    let (builder, _engine_id) = kyron::runtime::RuntimeBuilder::new()
        .with_engine(ExecutionEngineBuilder::new().task_queue_size(256).workers(2));

    let mut runtime = builder.build().unwrap();

    // Build Orchestration
    let mut orch = Orchestration::new()
        .add_design(braking_component_design().expect("Failed to create design"))
        .design_done();

    // Deployment part - specify event details
    let mut deployment = orch.get_deployment_mut();

    // Bind design event to the system event
    deployment
        .bind_events_as_global("ADASEmergencyBrakeEvent", &["BrakeEvent".into()])
        .expect("Failed to specify event");

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
