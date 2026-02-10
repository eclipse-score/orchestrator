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
use serde::Deserialize;
use serde_json::Value;
use test_scenarios_rust::scenario::Scenario;

#[derive(Deserialize, Debug)]
struct TestInput {
    graph_name: Option<String>,
}

impl TestInput {
    pub fn new(input: &str) -> Self {
        let v: Value = serde_json::from_str(input).expect("Failed to parse input string");
        serde_json::from_value(v["test"].clone()).expect("Failed to parse \"test\" field")
    }
}

struct GraphHandler;

impl GraphHandler {
    fn graph_two_nodes() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphTwoNodes".into(), DesignConfig::default());
        design.add_program("GraphTwoNodesProgram", move |design_instance, builder| {
            let mut graph_builder = LocalGraphActionBuilder::new();
            let n0 = graph_builder.add_node(JustLogAction::new("node0"));
            let n1 = graph_builder.add_node(JustLogAction::new("node1"));
            builder.with_run_action(graph_builder.add_edges(n0, &[n1]).build(design_instance));
            Ok(())
        });
        Ok(design)
    }

    fn graph_no_edges() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphNoEdges".into(), DesignConfig::default());
        design.add_program("GraphNoEdgesProgram", move |design_instance, builder| {
            let mut graph_builder = LocalGraphActionBuilder::new();
            graph_builder.add_node(JustLogAction::new("node1"));
            graph_builder.add_node(JustLogAction::new("node0"));
            builder.with_run_action(graph_builder.build(design_instance));
            Ok(())
        });
        Ok(design)
    }

    fn graph_one_node() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphOneNode".into(), DesignConfig::default());
        design.add_program("GraphOneNodeProgram", move |design_instance, builder| {
            let mut graph_builder = LocalGraphActionBuilder::new();
            graph_builder.add_node(JustLogAction::new("node0"));
            builder.with_run_action(graph_builder.build(design_instance));
            Ok(())
        });
        Ok(design)
    }

    fn graph_empty_edges() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphEmptyEdges".into(), DesignConfig::default());
        design.add_program("GraphEmptyEdgesProgram", move |design_instance, builder| {
            let mut graph_builder = LocalGraphActionBuilder::new();
            let n0 = graph_builder.add_node(JustLogAction::new("node0"));
            let n1 = graph_builder.add_node(JustLogAction::new("node1"));
            let n2 = graph_builder.add_node(JustLogAction::new("node2"));
            builder.with_run_action(
                graph_builder
                    .add_edges(n0, &[])
                    .add_edges(n1, &[])
                    .add_edges(n2, &[])
                    .build(design_instance),
            );
            Ok(())
        });
        Ok(design)
    }

    fn graph_multiple_edges() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphMultipleEdges".into(), DesignConfig::default());
        design.add_program("GraphMultipleEdgesProgram", move |design_instance, builder| {
            let mut graph_builder = LocalGraphActionBuilder::new();
            let n0 = graph_builder.add_node(JustLogAction::new("node0"));
            let n1 = graph_builder.add_node(JustLogAction::new("node1"));
            let n2 = graph_builder.add_node(JustLogAction::new("node2"));
            let n3 = graph_builder.add_node(JustLogAction::new("node3"));
            let n4 = graph_builder.add_node(JustLogAction::new("node4"));
            builder.with_run_action(
                graph_builder
                    .add_edges(n0, &[n1, n2, n3, n4])
                    .add_edges(n1, &[n3])
                    .add_edges(n2, &[n3, n4])
                    .add_edges(n3, &[n4])
                    .build(design_instance),
            );
            Ok(())
        });
        Ok(design)
    }

    fn graph_cube() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphCube".into(), DesignConfig::default());
        design.add_program("GraphCubeProgram", move |design_instance, builder| {
            let mut graph_builder = LocalGraphActionBuilder::new();
            let n0 = graph_builder.add_node(JustLogAction::new("node0"));
            let n1 = graph_builder.add_node(JustLogAction::new("node1"));
            let n2 = graph_builder.add_node(JustLogAction::new("node2"));
            let n3 = graph_builder.add_node(JustLogAction::new("node3"));
            let n4 = graph_builder.add_node(JustLogAction::new("node4"));
            let n5 = graph_builder.add_node(JustLogAction::new("node5"));
            let n6 = graph_builder.add_node(JustLogAction::new("node6"));
            let n7 = graph_builder.add_node(JustLogAction::new("node7"));
            builder.with_run_action(
                graph_builder
                    .add_edges(n0, &[n1, n2, n4])
                    .add_edges(n1, &[n3, n5])
                    .add_edges(n2, &[n3, n6])
                    .add_edges(n3, &[n7])
                    .add_edges(n4, &[n5, n6])
                    .add_edges(n5, &[n7])
                    .add_edges(n6, &[n7])
                    .build(design_instance),
            );
            Ok(())
        });
        Ok(design)
    }

    fn graph_parallel_flows() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphParallelFlows".into(), DesignConfig::default());
        design.add_program("GraphParallelFlowsProgram", move |design_instance, builder| {
            let mut graph_builder = LocalGraphActionBuilder::new();
            let n0 = graph_builder.add_node(JustLogAction::new("node0"));
            let n1 = graph_builder.add_node(JustLogAction::new("node1"));
            let n2 = graph_builder.add_node(JustLogAction::new("node2"));
            let n3 = graph_builder.add_node(JustLogAction::new("node3"));
            let n4 = graph_builder.add_node(JustLogAction::new("node4"));
            let n5 = graph_builder.add_node(JustLogAction::new("node5"));
            builder.with_run_action(
                graph_builder
                    .add_edges(n0, &[n1])
                    .add_edges(n1, &[n2])
                    .add_edges(n3, &[n4])
                    .add_edges(n4, &[n5])
                    .build(design_instance),
            );
            Ok(())
        });
        Ok(design)
    }

    fn graph_loop() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphLoop".into(), DesignConfig::default());
        design.add_program("GraphLoopProgram", move |design_instance, builder| {
            builder.with_run_action({
                let mut graph_builder = LocalGraphActionBuilder::new();
                let n0 = graph_builder.add_node(JustLogAction::new("node0"));
                let n1 = graph_builder.add_node(JustLogAction::new("node1"));
                graph_builder
                    .add_edges(n0, &[n1])
                    .add_edges(n1, &[n0])
                    .build(design_instance)
            });
            Ok(())
        });
        Ok(design)
    }

    fn graph_self_loop() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphSelfLoop".into(), DesignConfig::default());
        design.add_program("GraphSelfLoopProgram", move |design_instance, builder| {
            let mut graph_builder = LocalGraphActionBuilder::new();
            let n0 = graph_builder.add_node(JustLogAction::new("node0"));
            let n1 = graph_builder.add_node(JustLogAction::new("node1"));
            builder.with_run_action(
                graph_builder
                    .add_edges(n0, &[n1])
                    .add_edges(n1, &[n1])
                    .build(design_instance),
            );
            Ok(())
        });
        Ok(design)
    }

    fn graph_not_enough_nodes() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphNotEnoughNodes".into(), DesignConfig::default());
        design.add_program("GraphNotEnoughNodesProgram", move |design_instance, builder| {
            let mut graph_builder = LocalGraphActionBuilder::new();
            let n0 = graph_builder.add_node(JustLogAction::new("node0"));
            let n1 = 1_usize;
            builder.with_run_action(graph_builder.add_edges(n0, &[n1]).build(design_instance));
            Ok(())
        });
        Ok(design)
    }

    fn graph_invalid_node() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphInvalidNode".into(), DesignConfig::default());
        design.add_program("GraphInvalidNodeProgram", move |design_instance, builder| {
            let mut graph_builder = LocalGraphActionBuilder::new();

            let n0 = graph_builder.add_node(JustLogAction::new("node0"));
            let n1 = graph_builder.add_node(JustLogAction::new("node1"));
            let n2 = 2_usize;
            builder.with_run_action(graph_builder.add_edges(n2, &[n0, n1]).build(design_instance));
            Ok(())
        });
        Ok(design)
    }

    fn graph_invalid_edge() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphInvalidEdge".into(), DesignConfig::default());
        design.add_program("GraphInvalidEdgeProgram", move |design_instance, builder| {
            let mut graph_builder = LocalGraphActionBuilder::new();
            let n0 = graph_builder.add_node(JustLogAction::new("node0"));
            let n1 = graph_builder.add_node(JustLogAction::new("node1"));
            let n2 = 2_usize;
            builder.with_run_action(graph_builder.add_edges(n0, &[n1, n2]).build(design_instance));
            Ok(())
        });
        Ok(design)
    }

    fn graph_duplicated_edge() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphDuplicatedEdge".into(), DesignConfig::default());
        design.add_program("GraphDuplicatedEdgeProgram", move |design_instance, builder| {
            let mut graph_builder = LocalGraphActionBuilder::new();
            let n0 = graph_builder.add_node(JustLogAction::new("node0"));
            let n1 = graph_builder.add_node(JustLogAction::new("node1"));
            let n2 = graph_builder.add_node(JustLogAction::new("node2"));
            builder.with_run_action(
                graph_builder
                    .add_edges(n0, &[n1])
                    .add_edges(n1, &[n2, n2])
                    .build(design_instance),
            );
            Ok(())
        });
        Ok(design)
    }
    fn graph_in_sequence() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphDesign".into(), DesignConfig::default());

        // Create a program with some actions
        design.add_program("GraphDesignProgram", move |design_instance, builder| {
            builder.with_run_action(
                SequenceBuilder::new()
                    .with_step({
                        let mut graph_builder1 = LocalGraphActionBuilder::new();
                        let n0 = graph_builder1.add_node(JustLogAction::new("node0"));
                        let n1 = graph_builder1.add_node(JustLogAction::new("node1"));

                        graph_builder1.add_edges(n0, &[n1]).build(design_instance)
                    })
                    .with_step({
                        let mut graph_builder2 = LocalGraphActionBuilder::new();
                        let n2 = graph_builder2.add_node(JustLogAction::new("node2"));
                        let n3 = graph_builder2.add_node(JustLogAction::new("node3"));

                        graph_builder2.add_edges(n2, &[n3]).build(design_instance)
                    })
                    .build(),
            );
            Ok(())
        });

        Ok(design)
    }
    fn graph_in_concurrency() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphDesign".into(), DesignConfig::default());

        // Create a program with some actions
        design.add_program("GraphDesignProgram", move |design_instance, builder| {
            builder.with_run_action(
                ConcurrencyBuilder::new()
                    .with_branch({
                        let mut graph_builder1 = LocalGraphActionBuilder::new();

                        let n0 = graph_builder1.add_node(JustLogAction::new("node0"));
                        let n1 = graph_builder1.add_node(JustLogAction::new("node1"));

                        graph_builder1.add_edges(n0, &[n1]).build(design_instance)
                    })
                    .with_branch({
                        let mut graph_builder2 = LocalGraphActionBuilder::new();
                        let n2 = graph_builder2.add_node(JustLogAction::new("node2"));
                        let n3 = graph_builder2.add_node(JustLogAction::new("node3"));
                        let n4 = graph_builder2.add_node(JustLogAction::new("node4"));
                        graph_builder2
                            .add_edges(n2, &[n3])
                            .add_edges(n3, &[n4])
                            .build(design_instance)
                    })
                    .build(design_instance),
            );
            Ok(())
        });

        Ok(design)
    }

    fn two_programs() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphDesign".into(), DesignConfig::default());

        // Create a program with some actions
        design.add_program("GraphDesignProgram", move |design_instance, builder| {
            builder.with_run_action({
                let mut graph_builder1 = LocalGraphActionBuilder::new();

                let n0 = graph_builder1.add_node(JustLogAction::new("node0"));
                let n1 = graph_builder1.add_node(JustLogAction::new("node1"));

                graph_builder1.add_edges(n0, &[n1]).build(design_instance)
            });
            Ok(())
        });
        design.add_program("GraphDesignProgram2", move |design_instance, builder| {
            builder.with_run_action({
                let mut graph_builder2 = LocalGraphActionBuilder::new();
                let n2 = graph_builder2.add_node(JustLogAction::new("node2"));
                let n3 = graph_builder2.add_node(JustLogAction::new("node3"));
                let n4 = graph_builder2.add_node(JustLogAction::new("node4"));
                graph_builder2
                    .add_edges(n2, &[n3])
                    .add_edges(n3, &[n4])
                    .build(design_instance)
            });
            Ok(())
        });

        Ok(design)
    }

    fn graph_with_dedicated() -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphDesign".into(), DesignConfig::default());

        let sync_tag_1 = design.register_invoke_fn("sync1".into(), generic_test_func!("sync1"))?;
        let sync_tag_2 = design.register_invoke_fn("sync2".into(), generic_test_func!("sync2"))?;
        let sync_tag_3 = design.register_invoke_fn("sync3".into(), generic_test_func!("sync3"))?;
        let sync_tag_4 = design.register_invoke_fn("sync4".into(), generic_test_func!("sync4"))?;
        let sync_tag_5 = design.register_invoke_fn("sync5".into(), generic_test_func!("sync5"))?;

        design.add_program("GraphDesignProgram", move |design_instance, builder| {
            builder.with_run_action({
                let mut graph_builder = LocalGraphActionBuilder::new();

                let n0 = graph_builder.add_node(JustLogAction::new("node0"));
                let n1 = graph_builder.add_node(Invoke::from_tag(&sync_tag_1, design_instance.config()));
                let n2 = graph_builder.add_node(Invoke::from_tag(&sync_tag_2, design_instance.config()));
                let n3 = graph_builder.add_node(Invoke::from_tag(&sync_tag_3, design_instance.config()));
                let n4 = graph_builder.add_node(Invoke::from_tag(&sync_tag_4, design_instance.config()));
                let n5 = graph_builder.add_node(Invoke::from_tag(&sync_tag_5, design_instance.config()));

                graph_builder
                    .add_edges(n0, &[n1])
                    .add_edges(n1, &[n2])
                    .add_edges(n2, &[n3])
                    .add_edges(n3, &[n4])
                    .add_edges(n4, &[n5])
                    .build(design_instance)
            });
            Ok(())
        });

        Ok(design)
    }
    pub fn choose_graph(name: &str) -> Result<Design, CommonErrors> {
        match name {
            // positive scenarios
            "two_nodes" => Self::graph_two_nodes(),
            "no_edges" => Self::graph_no_edges(),
            "one_node" => Self::graph_one_node(),
            "multiple_edges" => Self::graph_multiple_edges(),
            "empty_edges" => Self::graph_empty_edges(),
            "cube" => Self::graph_cube(),
            "parallel_flows" => Self::graph_parallel_flows(),
            // negative scenarios
            "loop" => Self::graph_loop(),
            "self_loop" => Self::graph_self_loop(),
            "not_enough_nodes" => Self::graph_not_enough_nodes(),
            "invalid_edge" => Self::graph_invalid_edge(),
            "invalid_node" => Self::graph_invalid_node(),
            "duplicated_edge" => Self::graph_duplicated_edge(),
            // integration scenarios
            "two_steps" => Self::graph_in_sequence(),
            "two_programs" => Self::two_programs(),
            "concurrency" => Self::graph_in_concurrency(),
            "dedicated" => Self::graph_with_dedicated(),
            _ => Err(CommonErrors::NotFound),
        }
    }
}

struct GraphProgram;

impl Scenario for GraphProgram {
    fn name(&self) -> &str {
        "graph_program"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let logic = TestInput::new(input);
        let graph_name = logic.graph_name.expect("graph_name is required in --input");

        let builder = Runtime::from_json(input)?;
        let mut rt = builder.build();

        let orch = Orchestration::new()
            .add_design(GraphHandler::choose_graph(&graph_name).expect("Failed to create design"))
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

struct IntegrationGraph;

impl Scenario for IntegrationGraph {
    fn name(&self) -> &str {
        "integration_graph"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let logic = TestInput::new(input);
        let graph_name = logic.graph_name.expect("graph_name is required in --input");

        let builder = Runtime::from_json(input)?;
        let mut rt = builder.build();

        let orch = Orchestration::new()
            .add_design(GraphHandler::choose_graph(&graph_name).expect("Failed to create design"))
            .design_done();

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();
        println!("Programs: {:?}", programs);

        rt.block_on(async move {
            for program in programs.iter_mut() {
                println!("{:?}", program);
                let _ = program.run_n(1).await;
            }
        });
        Ok(())
    }
}

struct DedicatedGraph;
impl Scenario for DedicatedGraph {
    fn name(&self) -> &str {
        "dedicated_graph"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let builder = Runtime::from_json(input)?;
        let mut rt = builder.build();

        let mut orch = Orchestration::new()
            .add_design(GraphHandler::graph_with_dedicated().expect("Failed to create design"))
            .design_done();

        // Bind to dedicated worker
        let mut deployment = orch.get_deployment_mut();
        deployment
            .bind_invoke_to_worker("sync1".into(), "dedicated_worker_0".into())
            .expect("Failed to bind invoke action to worker");
        deployment
            .bind_invoke_to_worker("sync2".into(), "dedicated_worker_0".into())
            .expect("Failed to bind invoke action to worker");
        deployment
            .bind_invoke_to_worker("sync3".into(), "dedicated_worker_0".into())
            .expect("Failed to bind invoke action to worker");
        deployment
            .bind_invoke_to_worker("sync4".into(), "dedicated_worker_0".into())
            .expect("Failed to bind invoke action to worker");
        deployment
            .bind_invoke_to_worker("sync5".into(), "dedicated_worker_0".into())
            .expect("Failed to bind invoke action to worker");

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();
        rt.block_on(async move {
            for program in programs.iter_mut() {
                println!("{:?}", program);
                let _ = program.run_n(1).await;
            }
        });
        Ok(())
    }
}

pub fn graph_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "graphs",
        vec![
            Box::new(GraphProgram),
            Box::new(IntegrationGraph),
            Box::new(DedicatedGraph),
        ],
        vec![],
    ))
}
