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
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::vec::Vec;

fn simple_checkpoint(id: &str) {
    info!(id = id);
}

fn catch_checkpoint(e: &HandlerErrors) {
    match e {
        HandlerErrors::UserErr(user_error) => {
            let error_code: u64 = **user_error;
            info!(
                id = "catch",
                error_code = error_code,
                "Caught unrecoverable user error: {user_error:?}"
            );
        },
        _ => {
            panic!("Unexpected error type: {e:?}");
        },
    }
}

fn recoverable_catch_checkpoint(e: &HandlerErrors, is_recoverable: bool) {
    match e {
        HandlerErrors::UserErr(user_error) => {
            let error_code: u64 = **user_error;
            let is_recoverable_str = if is_recoverable { "recoverable" } else { "unrecoverable" };
            info!(
                id = "catch_recoverable",
                error_code = error_code,
                is_recoverable = is_recoverable,
                "Caught {is_recoverable_str} user error: {user_error:?}. Returning {is_recoverable}"
            );
        },
        _ => {
            panic!("Unexpected error type: {e:?}");
        },
    }
}

async fn generic_user_error_task(name: &str, error_code: u64) -> InvokeResult {
    info!(id = name, error_code = error_code);
    UserErrValue::from(error_code).into()
}

macro_rules! user_error_task {
    ($name:expr, $error_code:expr) => {
        move || generic_user_error_task($name, $error_code)
    };
}

async fn generic_user_error_task_own(name: String, error_code: u64) -> InvokeResult {
    info!(id = name.as_str(), error_code = error_code);
    UserErrValue::from(error_code).into()
}

macro_rules! user_error_task_owned {
    ($name:expr, $error_code:expr) => {
        move || generic_user_error_task_own($name.clone(), $error_code)
    };
}

async fn generic_just_log_task(name: &str) -> InvokeResult {
    simple_checkpoint(name);
    Ok(())
}

macro_rules! just_log_task {
    ($name:expr) => {
        move || generic_just_log_task($name)
    };
}

async fn generic_just_log_task_own(name: String) -> InvokeResult {
    simple_checkpoint(name.as_str());
    Ok(())
}

macro_rules! just_log_task_owned {
    ($name:expr) => {
        move || generic_just_log_task_own($name.clone())
    };
}

async fn generic_delayed_user_error_task_own(name: String, error_code: u64) -> InvokeResult {
    std::thread::sleep(std::time::Duration::from_millis(2000));
    info!(id = name.as_str(), error_code = error_code);
    UserErrValue::from(error_code).into()
}

macro_rules! delayed_user_error_task_owned {
    ($name:expr, $error_code:expr) => {
        move || generic_delayed_user_error_task_own($name.clone(), $error_code)
    };
}

#[derive(Serialize, Deserialize, Debug)]
struct DesignTypeTestInput {
    design_type: String,
    error_code: u64,
    run_count: u64,
}

impl DesignTypeTestInput {
    pub fn new(input: &str) -> Self {
        let v: Value = serde_json::from_str(input).expect("Failed to parse input string");
        serde_json::from_value(v["test"].clone()).expect("Failed to parse \"test\" field")
    }
}

pub struct CatchSequenceUserError;

impl CatchSequenceUserError {
    fn unrecoverable_error_design(&self, error_code: u64) -> Result<Design, CommonErrors> {
        let mut design = Design::new("unrecoverable_error_design".into(), DesignConfig::default());

        let user_error_tag = design.register_invoke_async(
            "user_error_task".into(),
            user_error_task!("user_error_task", error_code),
        )?;
        let log_after_error_tag =
            design.register_invoke_async("log_after_error_task".into(), just_log_task!("log_after_error_task"))?;

        design.add_program("catch_program", move |design, builder| {
            builder.with_run_action(
                CatchBuilder::new(
                    ErrorFilter::UserErrors.into(),
                    SequenceBuilder::new()
                        .with_step(Invoke::from_tag(&user_error_tag, design.config()))
                        .with_step(Invoke::from_tag(&log_after_error_tag, design.config()))
                        .build(),
                )
                .catch(|e| {
                    catch_checkpoint(&e);
                })
                .build(design),
            );

            Ok(())
        });

        Ok(design)
    }

    fn recoverable_error_design(&self, error_code: u64, is_recoverable: bool) -> Result<Design, CommonErrors> {
        let mut design = Design::new("recoverable_error_false_design".into(), DesignConfig::default());

        let user_error_tag = design.register_invoke_async(
            "user_error_task".into(),
            user_error_task!("user_error_task", error_code),
        )?;
        let log_after_error_tag =
            design.register_invoke_async("log_after_error_task".into(), just_log_task!("log_after_error_task"))?;
        let log_after_catch_tag =
            design.register_invoke_async("log_after_catch_task".into(), just_log_task!("log_after_catch_task"))?;

        design.add_program("catch_program", move |design, builder| {
            builder.with_run_action(
                SequenceBuilder::new()
                    .with_step(
                        CatchBuilder::new(
                            ErrorFilter::UserErrors.into(),
                            SequenceBuilder::new()
                                .with_step(Invoke::from_tag(&user_error_tag, design.config()))
                                .with_step(Invoke::from_tag(&log_after_error_tag, design.config()))
                                .build(),
                        )
                        .catch_recoverable(move |e| {
                            recoverable_catch_checkpoint(&e, is_recoverable);
                            is_recoverable
                        })
                        .build(design),
                    )
                    .with_step(Invoke::from_tag(&log_after_catch_tag, design.config()))
                    .build(),
            );

            Ok(())
        });

        Ok(design)
    }
}

impl Scenario for CatchSequenceUserError {
    fn name(&self) -> &str {
        "sequence_user_error"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let logic = DesignTypeTestInput::new(input);

        let orch = match logic.design_type.as_str() {
            "unrecoverable" => Orchestration::new()
                .add_design(
                    self.unrecoverable_error_design(logic.error_code)
                        .expect("Failed to create unrecoverable design"),
                )
                .design_done(),
            "recoverable_false" => Orchestration::new()
                .add_design(
                    self.recoverable_error_design(logic.error_code, false)
                        .expect("Failed to create recoverable_false design"),
                )
                .design_done(),
            "recoverable_true" => Orchestration::new()
                .add_design(
                    self.recoverable_error_design(logic.error_code, true)
                        .expect("Failed to create recoverable_true design"),
                )
                .design_done(),
            _ => return Err("Unknown design type".to_string()),
        };

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        rt.block_on(async move {
            let mut program = programs.pop().expect("Failed to pop program");
            let _ = program.run_n(logic.run_count as usize).await;
        });

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ErrorCodeTestInput {
    error_code: u64,
}

impl ErrorCodeTestInput {
    pub fn new(input: &str) -> Self {
        let v: Value = serde_json::from_str(input).expect("Failed to parse input string");
        serde_json::from_value(v["test"].clone()).expect("Failed to parse \"test\" field")
    }
}

pub struct CatchNestedSequenceUserError;

impl CatchNestedSequenceUserError {
    fn create_design(&self, error_code: u64) -> Result<Design, CommonErrors> {
        let mut design = Design::new("nested_catch_design".into(), DesignConfig::default());

        let user_error_tag = design.register_invoke_async(
            "user_error_task".into(),
            user_error_task!("user_error_task", error_code),
        )?;
        let just_log_tag = design.register_invoke_async("just_log_task".into(), just_log_task!("just_log_task"))?;

        design.add_program("catch_program", move |design, builder| {
            builder.with_run_action(
                CatchBuilder::new(
                    ErrorFilter::UserErrors.into(),
                    SequenceBuilder::new()
                        .with_step(
                            SequenceBuilder::new()
                                .with_step(Invoke::from_tag(&user_error_tag, design.config()))
                                .with_step(Invoke::from_tag(&just_log_tag, design.config()))
                                .build(),
                        )
                        .with_step(Invoke::from_tag(&just_log_tag, design.config()))
                        .build(),
                )
                .catch(|e| {
                    catch_checkpoint(&e);
                })
                .build(design),
            );

            Ok(())
        });

        Ok(design)
    }
}

impl Scenario for CatchNestedSequenceUserError {
    fn name(&self) -> &str {
        "nested_sequence_user_error"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let logic = ErrorCodeTestInput::new(input);

        let orch = Orchestration::new()
            .add_design(self.create_design(logic.error_code).expect("Failed to create design"))
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
struct ConcurrencyTestInput {
    concurrent_valid_tasks: Vec<String>,
    error_code: u64,
}

impl ConcurrencyTestInput {
    pub fn new(input: &str) -> Self {
        let v: Value = serde_json::from_str(input).expect("Failed to parse input string");
        serde_json::from_value(v["test"].clone()).expect("Failed to parse \"test\" field")
    }
}

pub struct CatchConcurrencyUserError;

impl CatchConcurrencyUserError {
    fn create_design(&self, valid_tasks: &[String], error_code: u64) -> Result<Design, CommonErrors> {
        let mut design = Design::new("concurrency_catch_design".into(), DesignConfig::default());

        let task_a_name = valid_tasks[0].clone();
        let task_b_name = valid_tasks[1].clone();
        let task_c_name = valid_tasks[2].clone();

        let user_error_tag = design.register_invoke_async(
            "user_error_task".into(),
            user_error_task!("user_error_task", error_code),
        )?;
        let just_log_a_tag =
            design.register_invoke_async(valid_tasks[0].clone().into(), just_log_task_owned!(task_a_name))?;
        let just_log_b_tag =
            design.register_invoke_async(valid_tasks[1].clone().into(), just_log_task_owned!(task_b_name))?;
        let just_log_c_tag =
            design.register_invoke_async(valid_tasks[2].clone().into(), just_log_task_owned!(task_c_name))?;
        let just_log_tag = design.register_invoke_async("just_log_task".into(), just_log_task!("just_log_task"))?;

        design.add_program("catch_program", move |design, builder| {
            builder.with_run_action(
                CatchBuilder::new(
                    ErrorFilter::UserErrors.into(),
                    SequenceBuilder::new()
                        .with_step(
                            ConcurrencyBuilder::new()
                                .with_branch(Invoke::from_tag(&just_log_a_tag, design.config()))
                                .with_branch(Invoke::from_tag(&user_error_tag, design.config()))
                                .with_branch(Invoke::from_tag(&just_log_b_tag, design.config()))
                                .with_branch(Invoke::from_tag(&just_log_c_tag, design.config()))
                                .build(design),
                        )
                        .with_step(Invoke::from_tag(&just_log_tag, design.config()))
                        .build(),
                )
                .catch(|e| {
                    catch_checkpoint(&e);
                })
                .build(design),
            );

            Ok(())
        });

        Ok(design)
    }
}

impl Scenario for CatchConcurrencyUserError {
    fn name(&self) -> &str {
        "concurrency_user_error"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let logic = ConcurrencyTestInput::new(input);
        let valid_tasks_len = logic.concurrent_valid_tasks.len();
        if valid_tasks_len != 3 {
            panic!("Test issue, expecting 3 valid tasks, got {valid_tasks_len}");
        }

        let orch = Orchestration::new()
            .add_design(
                self.create_design(&logic.concurrent_valid_tasks, logic.error_code)
                    .expect("Failed to create design"),
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

pub struct CatchNestedConcurrencyUserError;

impl CatchNestedConcurrencyUserError {
    fn create_design(&self, valid_tasks: &[String], error_code: u64) -> Result<Design, CommonErrors> {
        let mut design = Design::new("nested_concurrency_catch_design".into(), DesignConfig::default());

        let task_a_name = valid_tasks[0].clone();
        let task_b_name = valid_tasks[1].clone();
        let task_c_name = valid_tasks[2].clone();

        let user_error_tag = design.register_invoke_async(
            "user_error_task".into(),
            user_error_task!("user_error_task", error_code),
        )?;
        let just_log_a_tag =
            design.register_invoke_async(valid_tasks[0].clone().into(), just_log_task_owned!(task_a_name))?;
        let just_log_b_tag =
            design.register_invoke_async(valid_tasks[1].clone().into(), just_log_task_owned!(task_b_name))?;
        let just_log_c_tag =
            design.register_invoke_async(valid_tasks[2].clone().into(), just_log_task_owned!(task_c_name))?;

        design.add_program("catch_program", move |design, builder| {
            builder.with_run_action(
                CatchBuilder::new(
                    ErrorFilter::UserErrors.into(),
                    ConcurrencyBuilder::new()
                        .with_branch(
                            ConcurrencyBuilder::new()
                                .with_branch(Invoke::from_tag(&user_error_tag, design.config()))
                                .with_branch(Invoke::from_tag(&just_log_a_tag, design.config()))
                                .build(design),
                        )
                        .with_branch(Invoke::from_tag(&just_log_b_tag, design.config()))
                        .with_branch(Invoke::from_tag(&just_log_c_tag, design.config()))
                        .build(design),
                )
                .catch(|e| {
                    catch_checkpoint(&e);
                })
                .build(design),
            );

            Ok(())
        });

        Ok(design)
    }
}

impl Scenario for CatchNestedConcurrencyUserError {
    fn name(&self) -> &str {
        "nested_concurrency_user_error"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let logic = ConcurrencyTestInput::new(input);
        let valid_tasks_len = logic.concurrent_valid_tasks.len();
        if valid_tasks_len != 3 {
            panic!("Test issue, expecting 3 valid tasks, got {valid_tasks_len}");
        }

        let orch = Orchestration::new()
            .add_design(
                self.create_design(&logic.concurrent_valid_tasks, logic.error_code)
                    .expect("Failed to create design"),
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

#[derive(Serialize, Deserialize, Debug)]
struct ErrorCodesTestInput {
    error_codes: Vec<u64>,
}

impl ErrorCodesTestInput {
    pub fn new(input: &str) -> Self {
        let v: Value = serde_json::from_str(input).expect("Failed to parse input string");
        serde_json::from_value(v["test"].clone()).expect("Failed to parse \"test\" field")
    }
}

pub struct CatchDoubleMixedUserError;

impl CatchDoubleMixedUserError {
    fn create_design(&self, error_codes: &[u64]) -> Result<Design, CommonErrors> {
        let mut design = Design::new("double_mixed_catch_design".into(), DesignConfig::default());

        let error_code_recoverable = error_codes[0];
        let error_task_a_name = format!("user_error_{}_task", error_code_recoverable);
        let error_code_unrecoverable = error_codes[1];
        let error_task_b_name = format!("user_error_{}_task", error_code_unrecoverable);

        let user_error_a_tag = design.register_invoke_async(
            error_task_a_name.clone().into(),
            delayed_user_error_task_owned!(error_task_a_name, error_code_recoverable),
        )?;
        let user_error_b_tag = design.register_invoke_async(
            error_task_b_name.clone().into(),
            user_error_task_owned!(error_task_b_name, error_code_unrecoverable),
        )?;
        let just_log_tag = design.register_invoke_async("just_log_task".into(), just_log_task!("just_log_task"))?;
        let log_after_catch_tag =
            design.register_invoke_async("log_after_catch_task".into(), just_log_task!("log_after_catch_task"))?;

        design.add_program("catch_program", move |design, builder| {
            builder.with_run_action(
                SequenceBuilder::new()
                    .with_step(
                        CatchBuilder::new(
                            ErrorFilter::UserErrors.into(),
                            ConcurrencyBuilder::new()
                                .with_branch(Invoke::from_tag(&user_error_a_tag, design.config()))
                                .with_branch(Invoke::from_tag(&user_error_b_tag, design.config()))
                                .with_branch(Invoke::from_tag(&just_log_tag, design.config()))
                                .build(design),
                        )
                        .catch_recoverable(move |e| match e {
                            HandlerErrors::UserErr(error_code) => {
                                let is_recoverable;
                                if error_code == error_code_recoverable.into() {
                                    is_recoverable = true;
                                } else if error_code == error_code_unrecoverable.into() {
                                    is_recoverable = false;
                                } else {
                                    panic!("Unexpected error_code type: {error_code:?}");
                                }

                                recoverable_catch_checkpoint(&e, is_recoverable);
                                is_recoverable
                            },
                            _ => {
                                panic!("Unexpected error type: {e:?}");
                            },
                        })
                        .build(design),
                    )
                    .with_step(Invoke::from_tag(&log_after_catch_tag, design.config()))
                    .build(),
            );

            Ok(())
        });

        Ok(design)
    }
}

impl Scenario for CatchDoubleMixedUserError {
    fn name(&self) -> &str {
        "double_mixed_user_error"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let logic = ErrorCodesTestInput::new(input);
        let error_codes_len = logic.error_codes.len();
        if error_codes_len != 2 {
            panic!("Test issue, expecting 2 error codes, got {error_codes_len}");
        }

        let orch = Orchestration::new()
            .add_design(self.create_design(&logic.error_codes).expect("Failed to create design"))
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

pub struct CatchDoubleRecoverableUserError;

impl CatchDoubleRecoverableUserError {
    fn create_design(&self, error_codes: &[u64]) -> Result<Design, CommonErrors> {
        let mut design = Design::new("double_recoverable_catch_design".into(), DesignConfig::default());

        let error_code_a = error_codes[0];
        let error_task_a_name = format!("user_error_{}_task", error_code_a);
        let error_code_b = error_codes[1];
        let error_task_b_name = format!("user_error_{}_task", error_code_b);

        let user_error_a_tag = design.register_invoke_async(
            error_task_a_name.clone().into(),
            delayed_user_error_task_owned!(error_task_a_name, error_code_a),
        )?;
        let user_error_b_tag = design.register_invoke_async(
            error_task_b_name.clone().into(),
            user_error_task_owned!(error_task_b_name, error_code_b),
        )?;
        let just_log_tag = design.register_invoke_async("just_log_task".into(), just_log_task!("just_log_task"))?;
        let log_after_catch_tag =
            design.register_invoke_async("log_after_catch_task".into(), just_log_task!("log_after_catch_task"))?;

        design.add_program("catch_program", move |design, builder| {
            builder.with_run_action(
                SequenceBuilder::new()
                    .with_step(
                        CatchBuilder::new(
                            ErrorFilter::UserErrors.into(),
                            ConcurrencyBuilder::new()
                                .with_branch(Invoke::from_tag(&user_error_a_tag, design.config()))
                                .with_branch(Invoke::from_tag(&user_error_b_tag, design.config()))
                                .with_branch(Invoke::from_tag(&just_log_tag, design.config()))
                                .build(design),
                        )
                        .catch_recoverable(move |e| {
                            let is_recoverable = true;
                            recoverable_catch_checkpoint(&e, is_recoverable);
                            is_recoverable
                        })
                        .build(design),
                    )
                    .with_step(Invoke::from_tag(&log_after_catch_tag, design.config()))
                    .build(),
            );

            Ok(())
        });

        Ok(design)
    }
}

impl Scenario for CatchDoubleRecoverableUserError {
    fn name(&self) -> &str {
        "double_recoverable_user_error"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let logic = ErrorCodesTestInput::new(input);
        let error_codes_len = logic.error_codes.len();
        if error_codes_len != 2 {
            panic!("Test issue, expecting 2 error codes, got {error_codes_len}");
        }

        let orch = Orchestration::new()
            .add_design(self.create_design(&logic.error_codes).expect("Failed to create design"))
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

pub struct DoubleCatchSequence;

impl DoubleCatchSequence {
    fn create_design(&self, error_code: u64) -> Result<Design, CommonErrors> {
        let mut design = Design::new("double_catch_design".into(), DesignConfig::default());

        let user_error_tag = design.register_invoke_async(
            "user_error_task".into(),
            user_error_task!("user_error_task", error_code),
        )?;
        let just_log_tag = design.register_invoke_async("just_log_task".into(), just_log_task!("just_log_task"))?;

        design.add_program("catch_program", move |design, builder| {
            builder.with_run_action(
                CatchBuilder::new(
                    ErrorFilter::UserErrors.into(),
                    SequenceBuilder::new()
                        .with_step(
                            CatchBuilder::new(
                                ErrorFilter::Timeouts.into(),
                                SequenceBuilder::new()
                                    .with_step(Invoke::from_tag(&user_error_tag, design.config()))
                                    .with_step(Invoke::from_tag(&just_log_tag, design.config()))
                                    .build(),
                            )
                            .catch(|e| {
                                info!(
                                    id = "unexpected_catch",
                                    "Caught user error while only timeout error filter is set! {e:?}"
                                );
                            })
                            .build(design),
                        )
                        .with_step(Invoke::from_tag(&just_log_tag, design.config()))
                        .build(),
                )
                .catch(|e| {
                    catch_checkpoint(&e);
                })
                .build(design),
            );

            Ok(())
        });

        Ok(design)
    }
}

impl Scenario for DoubleCatchSequence {
    fn name(&self) -> &str {
        "per_nested_sequence"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let logic = ErrorCodeTestInput::new(input);

        let orch = Orchestration::new()
            .add_design(self.create_design(logic.error_code).expect("Failed to create design"))
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
