use crate::internals::runtime_helper::Runtime;
use test_scenarios_rust::scenario::Scenario;

use super::*;
use kyron_foundation::prelude::*;
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
};

pub struct SingleConcurrency;

fn single_concurrency_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("SingleConcurrency".into(), DesignConfig::default());

    let t1_tag = design.register_invoke_fn("Function1".into(), generic_test_func!("Function1"))?;
    let t2_tag = design.register_invoke_fn("Function2".into(), generic_test_func!("Function2"))?;
    let t3_tag = design.register_invoke_fn("Function3".into(), generic_test_func!("Function3"))?;

    // Create a program with actions
    design.add_program(file!(), move |design, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(
                    ConcurrencyBuilder::new()
                        .with_branch(Invoke::from_tag(&t1_tag, design.config()))
                        .with_branch(Invoke::from_tag(&t2_tag, design.config()))
                        .with_branch(Invoke::from_tag(&t3_tag, design.config()))
                        .build(design),
                )
                .with_step(JustLogAction::new("FinishAction"))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks Concurrency Functions
impl Scenario for SingleConcurrency {
    fn name(&self) -> &str {
        "single"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();

        // Build Orchestration
        let orch = Orchestration::new()
            .add_design(single_concurrency_design().expect("Failed to create design"))
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

pub struct MultipleConcurrency;

fn multiple_concurrency_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("MultipleConcurrency".into(), DesignConfig::default());

    let t1_tag = design.register_invoke_fn("Function1".into(), generic_test_func!("Function1"))?;
    let t2_tag = design.register_invoke_fn("Function2".into(), generic_test_func!("Function2"))?;
    let t3_tag = design.register_invoke_fn("Function3".into(), generic_test_func!("Function3"))?;
    let t4_tag = design.register_invoke_fn("Function4".into(), generic_test_func!("Function4"))?;
    let t5_tag = design.register_invoke_fn("Function5".into(), generic_test_func!("Function5"))?;
    let t6_tag = design.register_invoke_fn("Function6".into(), generic_test_func!("Function6"))?;
    // Create a program with actions
    design.add_program(file!(), move |design, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(
                    ConcurrencyBuilder::new()
                        .with_branch(Invoke::from_tag(&t1_tag, design.config()))
                        .with_branch(Invoke::from_tag(&t2_tag, design.config()))
                        .with_branch(Invoke::from_tag(&t3_tag, design.config()))
                        .build(design),
                )
                .with_step(JustLogAction::new("IntermediateAction"))
                .with_step(
                    ConcurrencyBuilder::new()
                        .with_branch(Invoke::from_tag(&t4_tag, design.config()))
                        .with_branch(Invoke::from_tag(&t5_tag, design.config()))
                        .with_branch(Invoke::from_tag(&t6_tag, design.config()))
                        .build(design),
                )
                .with_step(JustLogAction::new("FinishAction"))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks Concurrency Functions
impl Scenario for MultipleConcurrency {
    fn name(&self) -> &str {
        "multiple"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();

        // Build Orchestration
        let orch = Orchestration::new()
            .add_design(multiple_concurrency_design().expect("Failed to create design"))
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

pub struct NestedConcurrency;

fn nested_concurrency_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("NestedConcurrency".into(), DesignConfig::default());

    let t1_tag = design.register_invoke_fn("OuterFunction1".into(), generic_test_func!("OuterFunction1"))?;
    let t2_tag = design.register_invoke_fn("InnerFunction1".into(), generic_test_func!("InnerFunction1"))?;
    let t3_tag = design.register_invoke_fn("InnerFunction2".into(), generic_test_func!("InnerFunction2"))?;
    let t4_tag = design.register_invoke_fn("OuterFunction2".into(), generic_test_func!("OuterFunction2"))?;

    // Create a program with actions
    design.add_program(file!(), move |design, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(
                    ConcurrencyBuilder::new()
                        .with_branch(Invoke::from_tag(&t1_tag, design.config()))
                        .with_branch(
                            ConcurrencyBuilder::new()
                                .with_branch(Invoke::from_tag(&t2_tag, design.config()))
                                .with_branch(Invoke::from_tag(&t3_tag, design.config()))
                                .build(design),
                        )
                        .with_branch(Invoke::from_tag(&t4_tag, design.config()))
                        .build(design),
                )
                .with_step(JustLogAction::new("FinishAction"))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks Concurrency Functions
impl Scenario for NestedConcurrency {
    fn name(&self) -> &str {
        "nested"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();

        // Build Orchestration
        let orch = Orchestration::new()
            .add_design(nested_concurrency_design().expect("Failed to create design"))
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
