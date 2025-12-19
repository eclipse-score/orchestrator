use crate::internals::runtime_helper::Runtime;
use test_scenarios_rust::scenario::Scenario;

use super::*;
use kyron_foundation::prelude::*;
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
};
pub struct SingleSequence;

fn single_sequence_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("SingleSequence".into(), DesignConfig::default());

    // Create a program with actions
    design.add_program(file!(), move |_design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(JustLogAction::new("Action1"))
                .with_step(JustLogAction::new("Action2"))
                .with_step(JustLogAction::new("Action3"))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks three actions in a single sequence execution
impl Scenario for SingleSequence {
    fn name(&self) -> &str {
        "single"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();

        // Build Orchestration
        let orch = Orchestration::new()
            .add_design(single_sequence_design().expect("Failed to create design"))
            .design_done();

        // Create programs
        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        // Put programs into runtime and run them
        rt.block_on(async move {
            let mut program = programs.pop().expect("Failed to pop program");
            let _ = program.run_n(1).await;
            info!("Program finished running.");
        });

        Ok(())
    }
}

pub struct NestedSequence;

fn nested_sequence_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("NestedSequence".into(), DesignConfig::default());

    // Create a program with actions
    design.add_program(file!(), move |_design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(JustLogAction::new("OuterAction1"))
                .with_step(
                    SequenceBuilder::new()
                        .with_step(JustLogAction::new("InnerAction1"))
                        .with_step(JustLogAction::new("InnerAction2"))
                        .build(),
                )
                .with_step(JustLogAction::new("OuterAction2"))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks actions in a inner and outer sequence execution
impl Scenario for NestedSequence {
    fn name(&self) -> &str {
        "nested"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();

        // Build Orchestration
        let orch = Orchestration::new()
            .add_design(nested_sequence_design().expect("Failed to create design"))
            .design_done();

        // Create programs
        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        // Put programs into runtime and run them
        rt.block_on(async move {
            let mut program = programs.pop().expect("Failed to pop program");
            let _ = program.run_n(1).await;
            info!("Program finished running.");
        });

        Ok(())
    }
}

pub struct AwaitSequence;

fn awaited_sequence_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("AwaitedSequence".into(), DesignConfig::default());

    let evt1 = design.register_event(Tag::from_str_static("Test_Event_1"))?;

    // Create a program with actions
    design.add_program(file!(), move |design, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(JustLogAction::new("Action1"))
                .with_step(
                    ConcurrencyBuilder::new()
                        .with_branch(
                            SequenceBuilder::new()
                                .with_step(JustLogAction::new("Action2"))
                                .with_step(JustLogAction::new("Action3"))
                                .build(),
                        )
                        .with_branch(
                            SequenceBuilder::new()
                                .with_step(JustLogAction::new("Action4"))
                                .with_step(SyncBuilder::from_tag(&evt1, design.config()))
                                .with_step(TriggerBuilder::from_tag(&evt1, design.config()))
                                .with_step(JustLogAction::new("Action5"))
                                .build(),
                        )
                        .build(design),
                )
                .with_step(JustLogAction::new("FinishAction"))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks three actions in a single sequence execution
impl Scenario for AwaitSequence {
    fn name(&self) -> &str {
        "await"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();

        // Build Orchestration
        let orch = Orchestration::new()
            .add_design(awaited_sequence_design().expect("Failed to create design"))
            .design_done();

        // Create programs
        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        // Put programs into runtime and run them
        rt.block_on(async move {
            let mut program = programs.pop().expect("Failed to pop program");
            let _ = program.run_n(1).await;
            info!("Program finished running.");
        });

        Ok(())
    }
}
