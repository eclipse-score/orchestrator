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
    actions::{ifelse::IfElse, invoke::Invoke, sequence::SequenceBuilder, sync::SyncBuilder, trigger::TriggerBuilder},
    api::{design::Design, Orchestration},
    common::DesignConfig,
    prelude::ActionExecError,
};

mod common;
use common::register_all_common_into_design;

fn program_component_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("ExampleDesign".into(), DesignConfig::default());
    register_all_common_into_design(&mut design)?;

    // This program will trigger an event if the condition is correctly evaluated to true, then sync on the event.
    // If the logic fails, then either an error will be produced, or the program will hang on the sync.
    design.add_program("ExampleProgram1", move |design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(IfElse::from_design(
                    "always_true_condition",
                    TriggerBuilder::from_design("Event1", design_instance),
                    Invoke::from_design("always_produce_error", design_instance),
                    design_instance,
                ))
                .with_step(SyncBuilder::from_design("Event1", design_instance))
                .with_step(Invoke::from_design("test1_sync_func", design_instance))
                .build(),
        );
        Ok(())
    });

    // This program will produce an error if the condition is correctly evaluated to false.
    design.add_program("ExampleProgram2", move |design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(IfElse::from_design(
                    "always_false_condition",
                    Invoke::from_design("test1_sync_func", design_instance),
                    Invoke::from_design("always_produce_error", design_instance),
                    design_instance,
                ))
                .with_step(Invoke::from_design("test2_sync_func", design_instance))
                .build(),
        );
        Ok(())
    });

    Ok(design)
}

fn main() {
    let _logger = LogAndTraceBuilder::new()
        .global_log_level(Level::TRACE)
        //.enable_tracing(TraceScope::AppScope)
        .enable_logging(true)
        .build()
        .expect("Failed to build tracing library");

    let mut orch = Orchestration::new()
        .add_design(program_component_design().expect("Failed to create design"))
        .design_done();
    orch.get_deployment_mut()
        .bind_events_as_local(&["Event1".into()])
        .expect("Failed to specify event");

    let mut program_manager = orch.into_program_manager().unwrap();
    let mut program1 = program_manager.get_program("ExampleProgram1").unwrap();
    let mut program2 = program_manager.get_program("ExampleProgram2").unwrap();

    let (builder, _engine_id) = kyron::runtime::RuntimeBuilder::new()
        .with_engine(ExecutionEngineBuilder::new().task_queue_size(256).workers(2));
    let mut runtime = builder.build().unwrap();
    runtime.block_on(async move {
        info!("Running program 1");
        let result = program1.run_n(1).await;
        assert_eq!(result, Ok(()));

        info!("Running program 2");
        let result = program2.run_n(1).await;
        assert_eq!(result, Err(ActionExecError::UserError(123.into())));

        info!("Programs finished running");
    });
}
