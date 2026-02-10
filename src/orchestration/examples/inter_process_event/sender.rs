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

use ::core::time::Duration;
use kyron::{futures::sleep, runtime::*};
use kyron_foundation::prelude::*;
use logging_tracing::{Level, LogAndTraceBuilder};
use orchestration::{
    api::{design::Design, Orchestration},
    common::{tag::Tag, DesignConfig},
    prelude::*,
};

pub fn collision_detection() -> InvokeResult {
    info!("Forward collision is imminent...");
    Ok(())
}

pub async fn some_work() -> InvokeResult {
    sleep::sleep(Duration::from_millis(100)).await;
    Ok(())
}

fn collision_detection_component_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("CollisionDetectionDesign".into(), DesignConfig::default());

    design.register_invoke_fn("collision_detection".into(), collision_detection)?;
    design.register_event(Tag::from_str_static("ForwardCollisionEvent"))?;
    design.register_invoke_async("some_work".into(), some_work)?;

    // Create a program with some actions
    design.add_program("CollisionDetectionProgram", move |design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(Invoke::from_design("collision_detection", design_instance))
                .with_step(TriggerBuilder::from_design("ForwardCollisionEvent", design_instance))
                .with_step(Invoke::from_design("some_work", design_instance))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

fn main() {
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
        .add_design(collision_detection_component_design().expect("Failed to create design"))
        .design_done();

    // Deployment part - specify event details
    let mut deployment = orch.get_deployment_mut();

    // Bind design event to the system event
    deployment
        .bind_events_as_global("ADASEmergencyBrakeEvent", &["ForwardCollisionEvent".into()])
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
