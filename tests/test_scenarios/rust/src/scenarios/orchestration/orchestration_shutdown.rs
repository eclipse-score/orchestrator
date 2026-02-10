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
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use std::vec::Vec;
use test_scenarios_rust::scenario::Scenario;

pub struct SingleProgramSingleShutdown;
pub struct TwoProgramsSingleShutdown;
pub struct TwoProgramsTwoShutdowns;
pub struct GetAllShutdowns;
pub struct OneProgramNotShut;
pub struct ShutdownBeforeStart;

// Helpers
#[derive(Clone)]
struct ActionCounter {
    run_cnt: Arc<AtomicUsize>,
    stop_flag: Arc<AtomicBool>,
}

// Designs
fn shutdown_design_with_counter(name: &str, shutdown_tag: Tag, counter: ActionCounter) -> Result<Design, CommonErrors> {
    let mut design = Design::new(name.into(), DesignConfig::default());
    let action_name = name.to_owned() + "::Action1";
    let stop_action_name = name.to_owned() + "::StopAction";

    // Register async for incrementing execution counter
    let name_str = name.to_owned();
    let execution_tag = design.register_invoke_async(format!("{name}::ExecutionCounter").into(), move || {
        let execution_counter = counter.run_cnt.clone();
        let name_str = name_str.clone();
        async move {
            execution_counter.fetch_add(1, Ordering::Release);
            info!("{}::run_cnt={}", name_str, execution_counter.load(Ordering::Acquire));
            Ok(())
        }
    })?;

    // Register async for setting stop flag
    let stop_tag = design.register_invoke_async(format!("{name}::StopFlag").into(), move || {
        let stop_flag = counter.stop_flag.clone();
        let stop_action_name = stop_action_name.clone();
        async move {
            stop_flag.store(true, Ordering::Release);
            info!("{} was executed", stop_action_name);
            Ok(())
        }
    })?;

    // Create a program with actions
    design.add_program(file!(), move |_design_instance, builder| {
        builder
            .with_run_action(
                SequenceBuilder::new()
                    .with_step(JustLogAction::new(action_name.clone()))
                    .with_step(Invoke::from_tag(&execution_tag, _design_instance.config()))
                    .build(),
            )
            .with_shutdown_event(shutdown_tag)
            .with_stop_action(
                Invoke::from_tag(&stop_tag, _design_instance.config()),
                std::time::Duration::from_secs(1),
            );
        Ok(())
    });

    Ok(design)
}

fn shutdown_design(name: &str, shutdown_tag: Tag) -> Result<Design, CommonErrors> {
    let mut design = Design::new(name.into(), DesignConfig::default());
    let action_name = name.to_owned() + "::Action1";
    let stop_action_name = name.to_owned() + "::StopAction";
    // Create a program with actions
    design.add_program(file!(), move |_design_instance, builder| {
        builder
            .with_run_action(
                SequenceBuilder::new()
                    .with_step(JustLogAction::new(action_name))
                    .build(),
            )
            .with_shutdown_event(shutdown_tag)
            .with_stop_action(JustLogAction::new(stop_action_name), std::time::Duration::from_secs(1));

        Ok(())
    });

    Ok(design)
}

fn infinite_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("InfiniteDesign".into(), DesignConfig::default());

    let pending_tag =
        design.register_invoke_async("PendingIndefinitely".into(), async || ::core::future::pending().await)?;

    // Create a program with actions
    design.add_program(file!(), move |_design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(JustLogAction::new("InfiniteDesign::Action1"))
                .with_step(Invoke::from_tag(&pending_tag, _design_instance.config()))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

// Scenarios
impl Scenario for SingleProgramSingleShutdown {
    fn name(&self) -> &str {
        "single_program_single_shutdown"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let shutdown_tag = Tag::from_str_static("ShutdownEvent");

        // Build Orchestration
        let mut orch = Orchestration::new()
            .add_design(shutdown_design("ShutdownDesign", shutdown_tag).expect("Failed to create design"))
            .design_done();

        // Deployment part - specify event details
        let mut deployment = orch.get_deployment_mut();
        deployment
            .bind_shutdown_event_as_global(shutdown_tag.tracing_str(), shutdown_tag)
            .expect("Failed to bind shutdown event");

        // Create programs
        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        // Get shutdown notifier
        let mut shutdown = program_manager
            .get_shutdown_notifier(shutdown_tag)
            .expect("Failed to get shutdown notifier");

        // Put programs into runtime and run them
        let handle = rt.spawn(async move {
            let mut program = programs.pop().expect("Failed to pop program");
            let _ = program.run().await;
        });

        // Execute shutdown and wait for all engines to finish
        debug!("Initiating shutdown...");
        let _ = shutdown.shutdown();

        handle.join();
        debug!("EXIT.");

        Ok(())
    }
}

impl Scenario for TwoProgramsSingleShutdown {
    fn name(&self) -> &str {
        "two_programs_single_shutdown"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let shutdown_tag = Tag::from_str_static("ShutdownEvent");
        let counter_1 = ActionCounter {
            run_cnt: Arc::new(AtomicUsize::new(0)),
            stop_flag: Arc::new(AtomicBool::new(false)),
        };
        let counter_2 = ActionCounter {
            run_cnt: Arc::new(AtomicUsize::new(0)),
            stop_flag: Arc::new(AtomicBool::new(false)),
        };

        // Build Orchestration
        let mut orch = Orchestration::new()
            .add_design(
                shutdown_design_with_counter("ShutdownDesign1", shutdown_tag, counter_1.clone())
                    .expect("Failed to create design 1"),
            )
            .add_design(
                shutdown_design_with_counter("ShutdownDesign2", shutdown_tag, counter_2.clone())
                    .expect("Failed to create design 2"),
            )
            .design_done();

        // Deployment part - specify event details
        let mut deployment = orch.get_deployment_mut();
        deployment
            .bind_shutdown_event_as_global(shutdown_tag.tracing_str(), shutdown_tag)
            .expect("Failed to bind shutdown event");

        // Create programs
        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        // Get shutdown notifier
        let mut shutdown = program_manager
            .get_shutdown_notifier(shutdown_tag)
            .expect("Failed to get shutdown notifier");

        // Put programs into runtime and run them
        let handle = rt.spawn(async move {
            let mut joiner = Vec::new();
            for program in programs.as_mut_slice() {
                joiner.push(program.run());
            }
            futures::future::join_all(joiner).await;
        });

        // Ensure programs are running in a loop
        // Execute shutdown and wait for all engines to finish
        loop {
            if counter_1.run_cnt.load(Ordering::Acquire) > 2 && counter_2.run_cnt.load(Ordering::Acquire) > 2 {
                debug!("Initiating shutdowns...");
                let _ = shutdown.shutdown();
                break;
            }
        }
        handle.join();
        debug!("EXIT.");

        Ok(())
    }
}

impl Scenario for TwoProgramsTwoShutdowns {
    fn name(&self) -> &str {
        "two_programs_two_shutdowns"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let shutdown_tag_1 = Tag::from_str_static("ShutdownEvent1");
        let shutdown_tag_2 = Tag::from_str_static("ShutdownEvent2");

        let counter_1 = ActionCounter {
            run_cnt: Arc::new(AtomicUsize::new(0)),
            stop_flag: Arc::new(AtomicBool::new(false)),
        };
        let counter_2 = ActionCounter {
            run_cnt: Arc::new(AtomicUsize::new(0)),
            stop_flag: Arc::new(AtomicBool::new(false)),
        };

        // Build Orchestration
        let mut orch = Orchestration::new()
            .add_design(
                shutdown_design_with_counter("ShutdownDesign1", shutdown_tag_1, counter_1.clone())
                    .expect("Failed to create design 1"),
            )
            .add_design(
                shutdown_design_with_counter("ShutdownDesign2", shutdown_tag_2, counter_2.clone())
                    .expect("Failed to create design 2"),
            )
            .design_done();

        // Deployment part - specify event details
        let mut deployment = orch.get_deployment_mut();
        deployment
            .bind_shutdown_event_as_local(shutdown_tag_1)
            .expect("Failed to bind shutdown event 1");
        deployment
            .bind_shutdown_event_as_local(shutdown_tag_2)
            .expect("Failed to bind shutdown event 2");

        // Create programs
        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        // Get shutdown notifiers
        let mut shutdown_1 = program_manager
            .get_shutdown_notifier(shutdown_tag_1)
            .expect("Failed to get shutdown notifier 1");

        let mut shutdown_2 = program_manager
            .get_shutdown_notifier(shutdown_tag_2)
            .expect("Failed to get shutdown notifier 2");

        // Put programs into runtime and run them
        let handle = rt.spawn(async move {
            let mut joiner = Vec::new();
            for program in programs.as_mut_slice() {
                joiner.push(program.run());
            }
            futures::future::join_all(joiner).await;
        });

        // Ensure shutdown order execution
        // Loop until program 2 shutdown flag is set
        debug!("Initiating shutdown 2...");
        let _ = shutdown_2.shutdown();
        loop {
            if counter_2.stop_flag.load(Ordering::Acquire) {
                debug!("Initiating shutdown 1...");
                let _ = shutdown_1.shutdown();
                break;
            }
        }

        handle.join();
        debug!("EXIT.");
        Ok(())
    }
}

impl Scenario for GetAllShutdowns {
    fn name(&self) -> &str {
        "two_programs_all_shutdowns"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let shutdown_tag_1 = Tag::from_str_static("ShutdownEvent1");
        let shutdown_tag_2 = Tag::from_str_static("ShutdownEvent2");

        // Build Orchestration
        let mut orch = Orchestration::new()
            .add_design(shutdown_design("ShutdownDesign1", shutdown_tag_1).expect("Failed to create design 1"))
            .add_design(shutdown_design("ShutdownDesign2", shutdown_tag_2).expect("Failed to create design 2"))
            .design_done();

        // Deployment part - specify event details
        let mut deployment = orch.get_deployment_mut();
        deployment
            .bind_shutdown_event_as_local(shutdown_tag_1)
            .expect("Failed to bind shutdown event 1");
        deployment
            .bind_shutdown_event_as_local(shutdown_tag_2)
            .expect("Failed to bind shutdown event 2");

        // Create programs
        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        // Get shutdown notifiers
        let mut shutdowns = program_manager
            .get_shutdown_all_notifier()
            .expect("Failed to get shutdown notifiers");

        // Put programs into runtime and run them
        let handle = rt.spawn(async move {
            let mut joiner = Vec::new();
            for program in programs.as_mut_slice() {
                joiner.push(program.run());
            }
            futures::future::join_all(joiner).await;
        });

        // Execute shutdown and wait for all engines to finish
        debug!("Initiating shutdown...");
        let _ = shutdowns.shutdown();
        handle.join();
        debug!("EXIT.");

        Ok(())
    }
}

impl Scenario for OneProgramNotShut {
    fn name(&self) -> &str {
        "one_program_not_shut"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let shutdown_tag_1 = Tag::from_str_static("ShutdownEvent1");
        let shutdown_tag_2 = Tag::from_str_static("ShutdownEvent2");

        // Build Orchestration
        let mut orch = Orchestration::new()
            .add_design(shutdown_design("ShutdownDesign1", shutdown_tag_1).expect("Failed to create design 1"))
            .add_design(shutdown_design("ShutdownDesign2", shutdown_tag_2).expect("Failed to create design 2"))
            .add_design(infinite_design().expect("Failed to create infinite design"))
            .design_done();

        // Deployment part - specify event details
        let mut deployment = orch.get_deployment_mut();
        deployment
            .bind_shutdown_event_as_local(shutdown_tag_1)
            .expect("Failed to bind shutdown event 1");
        deployment
            .bind_shutdown_event_as_local(shutdown_tag_2)
            .expect("Failed to bind shutdown event 2");

        // Create programs
        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        // Get shutdown notifiers
        let mut shutdowns = program_manager
            .get_shutdown_all_notifier()
            .expect("Failed to get shutdown notifiers");

        // Put programs into runtime and run them
        let handle = rt.spawn(async move {
            let mut joiner = Vec::new();
            for program in programs.as_mut_slice() {
                joiner.push(program.run());
            }
            futures::future::join_all(joiner).await;
        });

        // Execute shutdown and wait for all engines to finish
        debug!("Initiating shutdown...");
        let _ = shutdowns.shutdown();
        handle.join();
        debug!("EXIT.");

        Ok(())
    }
}

impl Scenario for ShutdownBeforeStart {
    fn name(&self) -> &str {
        "before_start"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let shutdown_tag = Tag::from_str_static("ShutdownEvent");

        // Build Orchestration
        let mut orch = Orchestration::new()
            .add_design(shutdown_design("ShutdownDesign", shutdown_tag).expect("Failed to create design"))
            .design_done();

        // Deployment part - specify event details
        let mut deployment = orch.get_deployment_mut();
        deployment
            .bind_shutdown_event_as_global(shutdown_tag.tracing_str(), shutdown_tag)
            .expect("Failed to bind shutdown event");

        // Create programs
        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        // Get shutdown notifier
        let mut shutdown = program_manager
            .get_shutdown_notifier(shutdown_tag)
            .expect("Failed to get shutdown notifier");

        // Execute shutdown before program start
        debug!("Initiating shutdown...");
        let _ = shutdown.shutdown();

        // Put programs into runtime and run them
        let handle = rt.spawn(async move {
            let mut program = programs.pop().expect("Failed to pop program");
            let _ = program.run().await;
        });

        // Wait for all engines to finish
        handle.join();
        debug!("EXIT.");

        Ok(())
    }
}
