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

pub mod camera_driver;
pub mod object_detection;

use camera_driver::CameraDriver;
use object_detection::ObjectDetection;

use kyron::{futures::sleep, *};
use kyron_foundation::prelude::*;
use logging_tracing::{Level, LogAndTraceBuilder};
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
    prelude::*,
};

use std::sync::{Arc, Mutex};

pub async fn timer_input() -> InvokeResult {
    info!("Start of 'timer_input_async' function.");

    //sleep for 100ms
    sleep::sleep(::core::time::Duration::from_millis(100)).await;

    info!("End of 'timer_input_async' function.");
    Ok(())
}

fn camera_driver_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("camera_driver_design".into(), DesignConfig::default());

    let cam_drv = Arc::new(Mutex::new(CameraDriver::new()));
    let t1_tag = design.register_invoke_method("read_input".into(), cam_drv.clone(), CameraDriver::read_input)?;
    let t2_tag = design.register_invoke_method("process".into(), cam_drv.clone(), CameraDriver::process)?;
    let t3_tag = design.register_invoke_method("write_output".into(), cam_drv.clone(), CameraDriver::write_output)?;

    design.register_event("timer_event".into())?;
    design.register_event("trigger_obj_det".into())?;

    design.add_program("camera_driver_design", move |design, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(SyncBuilder::from_design("timer_event", design))
                .with_step(Invoke::from_tag(&t1_tag, design.config()))
                .with_step(Invoke::from_tag(&t2_tag, design.config()))
                .with_step(Invoke::from_tag(&t3_tag, design.config()))
                .with_step(TriggerBuilder::from_design("trigger_obj_det", design))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

fn timer_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("timer_design".into(), DesignConfig::default());

    let t1_tag = design.register_invoke_async("timer_input".into(), timer_input)?;

    design.register_event("timer_event".into())?;
    design.register_event("trigger_obj_det".into())?;

    design.add_program("timer_design", move |design, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(Invoke::from_tag(&t1_tag, design.config()))
                .with_step(TriggerBuilder::from_design("timer_event", design))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

fn obj_det_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("obj_det_design".into(), DesignConfig::default());

    let obj_det = Arc::new(Mutex::new(ObjectDetection::new()));
    let t1_tag = design.register_invoke_method(
        "pre_processing".into(),
        obj_det.clone(),
        ObjectDetection::pre_processing,
    )?;
    let t2_tag = design.register_invoke_method("drive_q1".into(), obj_det.clone(), ObjectDetection::drive_q1)?;
    let t3_tag = design.register_invoke_method("drive_q2".into(), obj_det.clone(), ObjectDetection::drive_q2)?;
    let t4_tag = design.register_invoke_method("drive_q3".into(), obj_det.clone(), ObjectDetection::drive_q3)?;
    let t5_tag =
        design.register_invoke_method("object_fusion".into(), obj_det.clone(), ObjectDetection::object_fusion)?;

    design.register_event("timer_event".into())?;
    design.register_event("trigger_obj_det".into())?;

    design.add_program("obj_det_design", move |design, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(SyncBuilder::from_design("trigger_obj_det", design))
                .with_step(Invoke::from_tag(&t1_tag, design.config()))
                .with_step(
                    ConcurrencyBuilder::new()
                        .with_branch(Invoke::from_tag(&t2_tag, design.config()))
                        .with_branch(Invoke::from_tag(&t3_tag, design.config()))
                        .with_branch(Invoke::from_tag(&t4_tag, design.config()))
                        .build(design),
                )
                .with_step(Invoke::from_tag(&t5_tag, design.config()))
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
        .with_engine(runtime::ExecutionEngineBuilder::new().task_queue_size(256).workers(3));
    let mut runtime = builder.build().unwrap();

    // Build Orchestration

    let mut orch = Orchestration::new()
        .add_design(camera_driver_design().expect("Failed to create design"))
        .add_design(obj_det_design().expect("Failed to create design"))
        .add_design(timer_design().expect("Failed to create design"))
        .design_done();

    // Deployment part - specify event details
    let mut deployment = orch.get_deployment_mut();

    // Mark user events as local one.
    deployment
        .bind_events_as_local(&["timer_event".into()])
        .expect("Failed to specify event");

    deployment
        .bind_events_as_local(&["trigger_obj_det".into()])
        .expect("Failed to specify event");

    // Create programs
    let mut program_manager = orch.into_program_manager().unwrap();
    let mut programs = program_manager.get_programs();

    // Put programs into runtime and run them
    runtime.block_on(async move {
        let mut program1 = programs.pop().unwrap();
        let mut program2 = programs.pop().unwrap();
        let mut program3 = programs.pop().unwrap();

        let h1 = kyron::spawn(async move {
            let _ = program1.run_n(3).await;
        });

        let h2 = kyron::spawn(async move {
            let _ = program2.run_n(3).await;
        });

        let h3 = kyron::spawn(async move {
            let _ = program3.run_n(3).await;
        });

        let _ = h1.await;
        let _ = h2.await;
        let _ = h3.await;

        info!("Programs finished running");
    });

    info!("Exit.");
}
