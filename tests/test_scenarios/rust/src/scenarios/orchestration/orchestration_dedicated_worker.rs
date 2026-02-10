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
use kyron_foundation::prelude::*;
use orchestration::api::design::Design;
use orchestration::api::Orchestration;
use orchestration::common::DesignConfig;
use test_scenarios_rust::scenario::Scenario;

fn dedicated_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("SingleSequence".into(), DesignConfig::default());

    let sync_tag_1 = design.register_invoke_fn("sync1".into(), generic_test_func!("sync1"))?;
    let sync_tag_2 = design.register_invoke_fn("sync2".into(), generic_test_func!("sync2"))?;

    let async_tag_1 = design.register_invoke_async("async1".into(), generic_async_test_func!("async1"))?;
    let async_tag_2 = design.register_invoke_async("async2".into(), generic_async_test_func!("async2"))?;

    // Create a program with actions
    design.add_program(file!(), move |_design_instance, builder| {
        builder.with_run_action(
            ConcurrencyBuilder::new()
                .with_branch(Invoke::from_tag(&async_tag_1, _design_instance.config()))
                .with_branch(Invoke::from_tag(&async_tag_2, _design_instance.config()))
                .with_branch(Invoke::from_tag(&sync_tag_1, _design_instance.config()))
                .with_branch(Invoke::from_tag(&sync_tag_2, _design_instance.config()))
                .build(_design_instance),
        );

        Ok(())
    });

    Ok(design)
}

struct DedicatedWorkerBindTags;

impl Scenario for DedicatedWorkerBindTags {
    fn name(&self) -> &str {
        "bind_tags"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let builder = Runtime::from_json(input)?;
        let mut rt = builder.build();

        let mut orch = Orchestration::new()
            .add_design(dedicated_design().expect("Failed to create design"))
            .design_done();
        let mut deployment = orch.get_deployment_mut();

        deployment
            .bind_invoke_to_worker("async1".into(), "dedicated_worker_0".into())
            .expect("Failed to bind invoke action to worker");
        deployment
            .bind_invoke_to_worker("async2".into(), "dedicated_worker_0".into())
            .expect("Failed to bind invoke action to worker");
        deployment
            .bind_invoke_to_worker("sync1".into(), "dedicated_worker_1".into())
            .expect("Failed to bind invoke action to worker");
        deployment
            .bind_invoke_to_worker("sync2".into(), "dedicated_worker_2".into())
            .expect("Failed to bind invoke action to worker");

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        rt.block_on(async move {
            let mut program = programs.pop().expect("Failed to pop program");
            let _ = program.run_n(1).await;
        });
        Ok(())
    }
}

struct DedicatedWorkerRepeatedAssignment;
impl Scenario for DedicatedWorkerRepeatedAssignment {
    fn name(&self) -> &str {
        "repeat_tag_assignment"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let builder = Runtime::from_json(input)?;
        let mut rt = builder.build();

        let mut orch = Orchestration::new()
            .add_design(dedicated_design().expect("Failed to create design"))
            .design_done();
        let mut deployment = orch.get_deployment_mut();

        deployment
            .bind_invoke_to_worker("async1".into(), "dedicated_worker_0".into())
            .expect("Failed to bind invoke action to worker");
        deployment
            .bind_invoke_to_worker("async1".into(), "dedicated_worker_1".into())
            .expect("Failed to bind invoke action to worker");

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        rt.block_on(async move {
            let mut program = programs.pop().expect("Failed to pop program");
            let _ = program.run_n(1).await;
        });
        Ok(())
    }
}

struct DedicatedWorkerNonExistentTag;
impl Scenario for DedicatedWorkerNonExistentTag {
    fn name(&self) -> &str {
        "assign_non_existent_tag"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let _builder = Runtime::from_json(input)?;

        let mut orch = Orchestration::new()
            .add_design(dedicated_design().expect("Failed to create design"))
            .design_done();
        let mut deployment = orch.get_deployment_mut();

        deployment
            .bind_invoke_to_worker("non_existent_tag".into(), "dedicated_worker_0".into())
            .expect("Failed to bind invoke action to worker");

        Ok(())
    }
}

struct DedicatedWorkerNonExistent;
impl Scenario for DedicatedWorkerNonExistent {
    fn name(&self) -> &str {
        "assign_to_non_existent_dedicated_worker"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let _builder = Runtime::from_json(input)?;

        let mut orch = Orchestration::new()
            .add_design(dedicated_design().expect("Failed to create design"))
            .design_done();
        let mut deployment = orch.get_deployment_mut();

        deployment
            .bind_invoke_to_worker("async1".into(), "non_existent_dedicated_worker".into())
            .expect("Failed to bind invoke action to worker");

        Ok(())
    }
}

struct DedicatedWorkerReuse;
impl Scenario for DedicatedWorkerReuse {
    fn name(&self) -> &str {
        "dedicated_works_on_regular"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let builder = Runtime::from_json(input)?;
        let mut rt = builder.build();

        let mut orch = Orchestration::new()
            .add_design(dedicated_design().expect("Failed to create design"))
            .design_done();
        let mut deployment = orch.get_deployment_mut();

        deployment
            .bind_invoke_to_worker("sync1".into(), "dedicated_worker_0".into())
            .expect("Failed to bind invoke action to worker");

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        rt.block_on(async move {
            let mut program = programs.pop().expect("Failed to pop program");
            let _ = program.run_n(1).await;
        });
        Ok(())
    }
}

pub fn dedicated_worker_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "dedicated_worker",
        vec![
            Box::new(DedicatedWorkerBindTags),
            Box::new(DedicatedWorkerReuse),
            Box::new(DedicatedWorkerRepeatedAssignment),
            Box::new(DedicatedWorkerNonExistentTag),
            Box::new(DedicatedWorkerNonExistent),
        ],
        vec![],
    ))
}
