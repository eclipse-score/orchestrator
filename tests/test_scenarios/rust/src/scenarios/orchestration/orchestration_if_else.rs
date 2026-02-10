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
use crate::internals::runtime_helper::Runtime;
use test_scenarios_rust::scenario::Scenario;

use super::*;
use kyron_foundation::prelude::*;
use orchestration::actions::ifelse::{IfElse, IfElseCondition};
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex};

async fn generic_just_log_task(name: &str) -> InvokeResult {
    info!(id = name);
    Ok(())
}

macro_rules! just_log_task {
    ($name:expr) => {
        move || generic_just_log_task($name)
    };
}

#[derive(Serialize, Deserialize, Debug)]
struct BasicTestInput {
    condition: bool,
}

impl BasicTestInput {
    pub fn new(input: &str) -> Self {
        let v: Value = serde_json::from_str(input).expect("Failed to parse input string");
        serde_json::from_value(v["test"].clone()).expect("Failed to parse \"test\" field")
    }
}

pub struct SettableCondition {
    condition: bool,
}

impl IfElseCondition for SettableCondition {
    fn compute(&self) -> bool {
        self.condition
    }
}
pub struct BasicIfElse;

impl BasicIfElse {
    fn if_else_design(condition: bool) -> Result<Design, CommonErrors> {
        let mut design = Design::new("BasicIfElse".into(), DesignConfig::default());

        let branch_true_tag = design.register_invoke_async("branch_true".into(), just_log_task!("true"))?;
        let branch_false_tag = design.register_invoke_async("branch_false".into(), just_log_task!("false"))?;
        let condition_tag = design.register_if_else_arc_mutex_condition(
            "condition".into(),
            Arc::new(Mutex::new(SettableCondition { condition })),
        )?;

        design.add_program("basic_if_else_program", move |design, builder| {
            builder.with_run_action(IfElse::from_tag(
                &condition_tag,
                Invoke::from_tag(&branch_true_tag, design.config()),
                Invoke::from_tag(&branch_false_tag, design.config()),
                design.config(),
            ));

            Ok(())
        });

        Ok(design)
    }
}

/// Checks IfElse action with true and false conditions
impl Scenario for BasicIfElse {
    fn name(&self) -> &str {
        "basic"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let logic = BasicTestInput::new(input);

        let orch = Orchestration::new()
            .add_design(Self::if_else_design(logic.condition).expect("Failed to create design"))
            .design_done();

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        rt.block_on(async move {
            let mut program = programs.pop().expect("Failed to pop program");
            let _ = program.run_n(1).await;
        });

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct NestedTestInput {
    outer_condition: bool,
    inner_condition: bool,
}

impl NestedTestInput {
    pub fn new(input: &str) -> Self {
        let v: Value = serde_json::from_str(input).expect("Failed to parse input string");
        serde_json::from_value(v["test"].clone()).expect("Failed to parse \"test\" field")
    }
}

pub struct NestedIfElse;

impl NestedIfElse {
    fn if_else_design(inner_condition: bool, outer_condition: bool) -> Result<Design, CommonErrors> {
        let mut design = Design::new("NestedIfElse".into(), DesignConfig::default());

        design.register_invoke_async("branch_true_true".into(), just_log_task!("true_true"))?;
        design.register_invoke_async("branch_true_false".into(), just_log_task!("true_false"))?;
        design.register_invoke_async("branch_false_true".into(), just_log_task!("false_true"))?;
        design.register_invoke_async("branch_false_false".into(), just_log_task!("false_false"))?;
        design.register_if_else_arc_condition(
            "outer_condition".into(),
            Arc::new(SettableCondition {
                condition: outer_condition,
            }),
        )?;
        design.register_if_else_condition(
            "inner_condition".into(),
            SettableCondition {
                condition: inner_condition,
            },
        )?;

        design.add_program("basic_if_else_program", move |design, builder| {
            builder.with_run_action(IfElse::from_design(
                "outer_condition",
                IfElse::from_design(
                    "inner_condition",
                    Invoke::from_design("branch_true_true", design),
                    Invoke::from_design("branch_true_false", design),
                    design,
                ),
                IfElse::from_design(
                    "inner_condition",
                    Invoke::from_design("branch_false_true", design),
                    Invoke::from_design("branch_false_false", design),
                    design,
                ),
                design,
            ));

            Ok(())
        });

        Ok(design)
    }
}

/// Checks nested IfElse action with true and false conditions
impl Scenario for NestedIfElse {
    fn name(&self) -> &str {
        "nested"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let logic = NestedTestInput::new(input);

        let orch = Orchestration::new()
            .add_design(
                Self::if_else_design(logic.inner_condition, logic.outer_condition).expect("Failed to create design"),
            )
            .design_done();

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        rt.block_on(async move {
            let mut program = programs.pop().expect("Failed to pop program");
            let _ = program.run_n(1).await;
        });

        Ok(())
    }
}
