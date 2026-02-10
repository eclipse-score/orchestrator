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
use futures::future;
use kyron_foundation::prelude::*;
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
};
use std::vec::Vec;

fn simple_checkpoint(id: &str) {
    info!(id = id);
}

fn location_checkpoint(id: &str, location: &str) {
    info!(id = id, location = location);
}

async fn blocking_sleep_task() -> InvokeResult {
    location_checkpoint("blocking_sleep_task", "begin");
    std::thread::sleep(std::time::Duration::from_secs(1));
    location_checkpoint("blocking_sleep_task", "end");
    Ok(())
}

async fn basic_task_a() -> InvokeResult {
    simple_checkpoint("basic_task_A");
    Ok(())
}

async fn basic_task_b() -> InvokeResult {
    simple_checkpoint("basic_task_B");
    Ok(())
}

pub struct OneTriggerOneSyncTwoPrograms;

fn trigger_sync_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("TriggerSequence".into(), DesignConfig::default());

    let blocking_sleep_task_tag = design.register_invoke_async("blocking_sleep".into(), blocking_sleep_task)?;
    let basic_task_tag = design.register_invoke_async("basic_task_a".into(), basic_task_a)?;
    let evt_sync = design.register_event(Tag::from_str_static("evt_sync"))?;

    design.add_program("trigger_program", move |design, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(Invoke::from_tag(&blocking_sleep_task_tag, design.config()))
                .with_step(TriggerBuilder::from_tag(&evt_sync, design.config()))
                .build(),
        );

        Ok(())
    });

    design.add_program("sync_program", move |design, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(SyncBuilder::from_design("evt_sync", design))
                .with_step(Invoke::from_tag(&basic_task_tag, design.config()))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks trigger and sync between two programs
impl Scenario for OneTriggerOneSyncTwoPrograms {
    fn name(&self) -> &str {
        "1_trigger_1_sync_2_programs"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();

        let mut orch = Orchestration::new()
            .add_design(trigger_sync_design().expect("Failed to create design"))
            .design_done();

        let mut deployment = orch.get_deployment_mut();
        deployment
            .bind_events_as_local(&["evt_sync".into()])
            .expect("Failed to specify event");

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        rt.block_on(async move {
            let mut joiner = Vec::new();
            for program in programs.as_mut_slice() {
                joiner.push(program.run_n(1));
            }

            future::join_all(joiner).await;
        });

        Ok(())
    }
}

pub struct OneTriggerTwoSyncsThreePrograms;

fn trigger_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("TriggerSequence".into(), DesignConfig::default());

    let blocking_sleep_task_tag = design.register_invoke_async("blocking_sleep".into(), blocking_sleep_task)?;
    let evt_sync = design.register_event(Tag::from_str_static("evt_sync"))?;

    design.add_program("trigger_program", move |design, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(Invoke::from_tag(&blocking_sleep_task_tag, design.config()))
                .with_step(TriggerBuilder::from_tag(&evt_sync, design.config()))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

fn sync_design_a() -> Result<Design, CommonErrors> {
    let mut design = Design::new("SyncA".into(), DesignConfig::default());

    let basic_task_tag = design.register_invoke_async("basic_task_a".into(), basic_task_a)?;
    let evt_sync = design.register_event(Tag::from_str_static("evt_sync"))?;

    design.add_program("sync_program_A", move |design, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(SyncBuilder::from_tag(&evt_sync, design.config()))
                .with_step(Invoke::from_tag(&basic_task_tag, design.config()))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

fn sync_design_b() -> Result<Design, CommonErrors> {
    let mut design = Design::new("SyncB".into(), DesignConfig::default());

    let basic_task_tag = design.register_invoke_async("basic_task".into(), basic_task_b)?;
    let evt_sync = design.register_event(Tag::from_str_static("evt_sync"))?;

    design.add_program("sync_program_B", move |design, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(SyncBuilder::from_tag(&evt_sync, design.config()))
                .with_step(Invoke::from_tag(&basic_task_tag, design.config()))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks trigger in one program and sync in other two programs
impl Scenario for OneTriggerTwoSyncsThreePrograms {
    fn name(&self) -> &str {
        "1_trigger_2_syncs_3_programs"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();

        let mut orch = Orchestration::new()
            .add_design(sync_design_a().expect("Failed to create design"))
            .add_design(sync_design_b().expect("Failed to create design"))
            .add_design(trigger_design().expect("Failed to create design"))
            .design_done();

        let mut deployment = orch.get_deployment_mut();
        deployment
            .bind_events_as_local(&["evt_sync".into()])
            .expect("Failed to specify event");

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        rt.block_on(async move {
            let mut joiner = Vec::new();
            for program in programs.as_mut_slice() {
                joiner.push(program.run_n(1));
            }

            future::join_all(joiner).await;
        });

        Ok(())
    }
}

pub struct TriggerAndSyncInNestedBranches;

fn nested_trigger_sync_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("TriggerSequence".into(), DesignConfig::default());

    let blocking_sleep_task_tag = design.register_invoke_async("blocking_sleep".into(), blocking_sleep_task)?;
    let basic_task_tag = design.register_invoke_async("basic_task".into(), basic_task_a)?;
    let evt_sync = design.register_event(Tag::from_str_static("evt_sync"))?;

    design.add_program("trigger_sync_program", move |design, builder| {
        builder.with_run_action(
            ConcurrencyBuilder::new()
                .with_branch(
                    SequenceBuilder::new()
                        .with_step(Invoke::from_tag(&blocking_sleep_task_tag, design.config()))
                        .with_step(TriggerBuilder::from_tag(&evt_sync, design.config()))
                        .build(),
                )
                .with_branch(
                    SequenceBuilder::new()
                        .with_step(SyncBuilder::from_design("evt_sync", design))
                        .with_step(Invoke::from_tag(&basic_task_tag, design.config()))
                        .build(),
                )
                .build(design),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks trigger and sync in the separate concurrency branches in a single program
impl Scenario for TriggerAndSyncInNestedBranches {
    fn name(&self) -> &str {
        "nested_branches"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();

        let mut orch = Orchestration::new()
            .add_design(nested_trigger_sync_design().expect("Failed to create design"))
            .design_done();

        let mut deployment = orch.get_deployment_mut();
        deployment
            .bind_events_as_local(&["evt_sync".into()])
            .expect("Failed to specify event");

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        rt.block_on(async move {
            let mut joiner = Vec::new();
            for program in programs.as_mut_slice() {
                joiner.push(program.run_n(1));
            }

            future::join_all(joiner).await;
        });

        Ok(())
    }
}

pub struct TriggerSyncOneAfterAnother;

fn trigger_sync_oaa_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("TriggerSequence".into(), DesignConfig::default());

    let basic_task_a_tag = design.register_invoke_async("basic_task_a".into(), basic_task_a)?;
    let basic_task_b_tag = design.register_invoke_async("basic_task_b".into(), basic_task_b)?;
    let evt_sync = design.register_event(Tag::from_str_static("evt_sync"))?;

    design.add_program("trigger_sync_program", move |design, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(Invoke::from_tag(&basic_task_a_tag, design.config()))
                .with_step(TriggerBuilder::from_tag(&evt_sync, design.config()))
                .with_step(SyncBuilder::from_tag(&evt_sync, design.config()))
                .with_step(Invoke::from_tag(&basic_task_b_tag, design.config()))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks trigger and sync as sequential steps in a single program
impl Scenario for TriggerSyncOneAfterAnother {
    fn name(&self) -> &str {
        "one_after_another"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();

        let mut orch = Orchestration::new()
            .add_design(trigger_sync_oaa_design().expect("Failed to create design"))
            .design_done();

        let mut deployment = orch.get_deployment_mut();
        deployment
            .bind_events_as_local(&["evt_sync".into()])
            .expect("Failed to specify event");

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        rt.block_on(async move {
            let mut joiner = Vec::new();
            for program in programs.as_mut_slice() {
                joiner.push(program.run_n(1));
            }

            future::join_all(joiner).await;
        });

        Ok(())
    }
}
