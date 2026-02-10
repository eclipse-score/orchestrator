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
use super::*;
use crate::internals::runtime_helper::Runtime;
use kyron::futures::sleep;
use kyron_foundation::prelude::*;
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use test_scenarios_rust::scenario::Scenario;
use tracing::info;

fn location_checkpoint(id: &str, location: &str) {
    info!(id = id, location = location);
}

async fn generic_non_blocking_sleep_task(name: String, delay_ms: u64) -> InvokeResult {
    location_checkpoint(name.as_str(), "begin");
    sleep::sleep(Duration::from_millis(delay_ms)).await;
    location_checkpoint(name.as_str(), "end");
    Ok(())
}
macro_rules! non_blocking_sleep_task {
    ($name:expr, $delay_ms:expr) => {
        move || generic_non_blocking_sleep_task($name, $delay_ms)
    };
}

// Simulate a CPU load with fibonacci calculation
fn generic_cpu_load_action(n: u64) -> InvokeResult {
    fn fib(x: u64) -> u64 {
        match x {
            0 => 0,
            1 => 1,
            n => fib(n - 1) + fib(n - 2),
        }
    }
    location_checkpoint("CpuLoad", "begin");
    let _ = fib(n);
    location_checkpoint("CpuLoad", "end");
    Ok(())
}

macro_rules! cpu_load_action {
    ($n:expr) => {
        || generic_cpu_load_action($n)
    };
}

#[derive(Serialize, Deserialize, Debug)]
struct TestInput {
    sleep_duration_ms: u64,
    run_count: usize,
    cpu_load: String,
}

impl TestInput {
    pub fn new(input: &str) -> Self {
        let v: Value = serde_json::from_str(input).expect("Failed to parse input string");
        serde_json::from_value(v["test"].clone()).expect("Failed to parse \"test\" field")
    }
}

pub struct SleepUnderLoad;

fn sleep_under_load(sleep_duration_ms: u64, cpu_load: String) -> Result<Design, CommonErrors> {
    let mut design = Design::new("SleepUnderLoad".into(), DesignConfig::default());

    // Register async actions as invoke functions and get tags
    let sleep1_tag = design.register_invoke_async(
        "Sleep1".into(),
        non_blocking_sleep_task!("Sleep1".to_string(), sleep_duration_ms),
    )?;

    let cpu_load_action = if cpu_load == "low" {
        cpu_load_action!(5)
    } else {
        cpu_load_action!(42)
    };
    let cpu_tag = design.register_invoke_fn("CpuLoadInput".into(), cpu_load_action)?;

    let sleep2_tag = design.register_invoke_async(
        "Sleep2".into(),
        non_blocking_sleep_task!("Sleep2".to_string(), sleep_duration_ms),
    )?;
    let sleep3_tag = design.register_invoke_async(
        "Sleep3".into(),
        non_blocking_sleep_task!("Sleep3".to_string(), sleep_duration_ms),
    )?;
    let sleep4_tag = design.register_invoke_async(
        "Sleep4".into(),
        non_blocking_sleep_task!("Sleep4".to_string(), sleep_duration_ms),
    )?;
    let sleep5_tag = design.register_invoke_async(
        "Sleep5".into(),
        non_blocking_sleep_task!("Sleep5".to_string(), sleep_duration_ms),
    )?;

    design.add_program(file!(), move |design, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(JustLogAction::new("StartAction"))
                .with_step(
                    ConcurrencyBuilder::new()
                        .with_branch(JustLogAction::new("Action1"))
                        .with_branch(Invoke::from_tag(&sleep1_tag, design.config()))
                        .with_branch(Invoke::from_tag(&cpu_tag, design.config()))
                        .with_branch(Invoke::from_tag(&sleep2_tag, design.config()))
                        .build(design),
                )
                .with_step(JustLogAction::new("IntermediateAction"))
                .with_step(
                    ConcurrencyBuilder::new()
                        .with_branch(JustLogAction::new("Action2"))
                        .with_branch(Invoke::from_tag(&sleep3_tag, design.config()))
                        .with_branch(Invoke::from_tag(&sleep4_tag, design.config()))
                        .build(design),
                )
                .with_step(Invoke::from_tag(&sleep5_tag, design.config()))
                .with_step(JustLogAction::new("FinishAction"))
                .build(),
        );
        Ok(())
    });

    Ok(design)
}

impl Scenario for SleepUnderLoad {
    fn name(&self) -> &str {
        "under_load"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let logic = TestInput::new(input);

        let design = sleep_under_load(logic.sleep_duration_ms, logic.cpu_load).expect("Failed to create design");

        let mut rt = Runtime::from_json(input)?.build();

        let orch = Orchestration::new().add_design(design).design_done();

        let mut program_manager: orchestration::api::OrchProgramManager =
            orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        rt.block_on(async move {
            let mut program = programs.pop().expect("Failed to pop program");
            let _ = program.run_n(logic.run_count).await;
        });
        Ok(())
    }
}
