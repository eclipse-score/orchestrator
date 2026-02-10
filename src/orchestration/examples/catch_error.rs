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
#![allow(unused_imports)]
use kyron::runtime::*;
use kyron_foundation::prelude::*;
use logging_tracing::{Level, LogAndTraceBuilder};
use orchestration::{
    actions::{catch::ErrorFilter, invoke::Invoke, sequence::SequenceBuilder},
    api::{design::Design, Orchestration},
    common::DesignConfig,
    prelude::{CatchBuilder, ConcurrencyBuilder},
    program::ProgramBuilder,
};

mod common;
use common::register_all_common_into_design;

fn catch_error_component_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("CatchErrorDesign".into(), DesignConfig::default());

    register_all_common_into_design(&mut design)?; // Register our common functions, events, etc

    // Create a program describing task chain
    design.add_program("CatchErrorProgramDesign", move |design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(
                    CatchBuilder::new(
                        ErrorFilter::UserErrors.into(),
                        ConcurrencyBuilder::new()
                            .with_branch(
                                SequenceBuilder::new()
                                    .with_step(Invoke::from_design("test2_sync_func", design_instance))
                                    .with_step(Invoke::from_design("test3_sync_func", design_instance))
                                    .build(),
                            )
                            .with_branch(
                                SequenceBuilder::new()
                                    .with_step(Invoke::from_design("test1_sync_func", design_instance))
                                    .with_step(Invoke::from_design("error_after_third_run", design_instance))
                                    .build(),
                            )
                            .build(design_instance),
                    )
                    .catch(|e| {
                        // Handle the error, e.g., log it or take some action
                        error!(
                            "Caught error: {:?}. This is not recoverable and we will stop execution",
                            e
                        );
                    })
                    .build(design_instance),
                )
                .with_step(Invoke::from_design("test4_sync_func", design_instance))
                .with_step(
                    CatchBuilder::new(
                        ErrorFilter::UserErrors.into(),
                        Invoke::from_design("always_produce_error", design_instance),
                    )
                    .catch_recoverable(|e| {
                        // Handle the error, e.g., log it or take some action
                        info!("Caught error: {:?}. This is catched and we continue executing", e);
                        true
                    })
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
    let (builder, _engine_id) = kyron::runtime::RuntimeBuilder::new()
        .with_engine(ExecutionEngineBuilder::new().task_queue_size(256).workers(2));
    let mut runtime = builder.build().unwrap();

    // Build Orchestration
    let orch = Orchestration::new()
        .add_design(catch_error_component_design().expect("Failed to create design"))
        .design_done();

    // Create programs
    let mut program_manager = orch.into_program_manager().unwrap();
    let mut programs = program_manager.get_programs();

    // Put programs into runtime and run them
    runtime.block_on(async move {
        let res = programs.pop().unwrap().run().await;
        info!("Program finished running with {:?}.", res);
    });
}
