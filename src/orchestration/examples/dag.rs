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

use core::time::Duration;

use kyron::prelude::*;
use kyron_foundation::prelude::*;
use logging_tracing::{Level, LogAndTraceBuilder};
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
    prelude::*,
};

mod common;
use common::register_all_common_into_design;

fn example_component_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("ExampleDesign".into(), DesignConfig::default());

    register_all_common_into_design(&mut design)?; // Register our common functions, events, etc

    design.register_event("cyclic_evt".into())?; // Register a timer event

    // Example of a DAG with 10 nodes
    // The graph structure is as follows from left to right:
    //
    // N1 ------> N3 --------> N6
    //  \                       \
    //   \------> N4 ---> N7 --> N9 ----> N10
    //   /                      /
    //  /                      /
    // N2 ------> N5 --------> N8
    let mut graph_builder = LocalGraphActionBuilder::new();
    // Addition of nodes can be in any order
    // Nodes will be sorted topologically when the graph is built based on the edges
    let n6 = graph_builder.add_node(Invoke::from_design("node6_sync_func", &design));
    let n7 = graph_builder.add_node(Invoke::from_design("node7_sync_func", &design));
    let n8 = graph_builder.add_node(Invoke::from_design("node8_sync_func", &design));
    let n9 = graph_builder.add_node(Invoke::from_design("node9_sync_func", &design));
    let n10 = graph_builder.add_node(Invoke::from_design("node10_sync_func", &design));
    let n1 = graph_builder.add_node(Invoke::from_design("node1_sync_func", &design));
    let n2 = graph_builder.add_node(Invoke::from_design("node2_sync_func", &design));
    let n3 = graph_builder.add_node(Invoke::from_design("node3_sync_func", &design));
    let n4 = graph_builder.add_node(Invoke::from_design("node4_sync_func", &design));
    let n5 = graph_builder.add_node(Invoke::from_design("node5_sync_func", &design));

    let graph_action = graph_builder
        .add_edges(n1, &[n3, n4])
        .add_edges(n2, &[n5, n4])
        .add_edges(n3, &[n6])
        .add_edges(n4, &[n7])
        .add_edges(n5, &[n8])
        .add_edges(n6, &[n9])
        .add_edges(n7, &[n9])
        .add_edges(n8, &[n9])
        .add_edges(n9, &[n10])
        .build(&design);

    // Create a program with some actions
    design.add_program("ExampleDesignProgram", move |design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(SyncBuilder::from_design("cyclic_evt", design_instance))
                .with_step(graph_action)
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
        .with_engine(ExecutionEngineBuilder::new().task_queue_size(256).workers(4));
    let mut runtime = builder.build().unwrap();

    // Build Orchestration
    let mut orch = Orchestration::new()
        .add_design(example_component_design().expect("Failed to create design"))
        .design_done();

    // Deployment part - specify event details
    let mut deployment = orch.get_deployment_mut();

    deployment
        .bind_events_as_timer(&["cyclic_evt".into()], Duration::from_secs(1))
        .expect("Failed to bind cycle event to timer");

    // Create programs
    let mut program_manager = orch.into_program_manager().unwrap();
    let mut programs = program_manager.get_programs();

    // Put programs into runtime and run them
    runtime.block_on(async move {
        let _ = programs.pop().unwrap().run_n(3).await;
        info!("Program finished running.");
    });

    info!("Exit.");
}
