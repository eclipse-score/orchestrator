use crate::internals::runtime_helper::Runtime;
use kyron::futures::reusable_box_future::ReusableBoxFuturePool;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use test_scenarios_rust::scenario::Scenario;

use core::time::Duration;
use kyron_foundation::prelude::*;
use orchestration::common::tag::Tag;

use super::*;
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct DemoLogic {
    cycle_duration_ms: u64,
}

impl DemoLogic {
    /// Creates a new DemoLogic from the "test" field in the input JSON.
    pub fn new(input: &str) -> Self {
        let v: Value = serde_json::from_str(input).expect("Failed to parse input string");
        serde_json::from_value(v["test"].clone()).expect("Failed to parse \"test\" field")
    }
}

pub struct JustLogAction {
    base: ActionBaseMeta,
    name: String,
}

impl JustLogAction {
    fn new(name: impl Into<String>) -> Box<JustLogAction> {
        const DEFAULT_TAG: &str = "integration::tests::just_log_action";

        Box::new(Self {
            base: ActionBaseMeta {
                tag: Tag::from_str_static(DEFAULT_TAG),
                reusable_future_pool: ReusableBoxFuturePool::for_value(1, Self::execute_impl("JustLogAction".into())),
            },
            name: name.into(),
        })
    }
    async fn execute_impl(name: String) -> ActionResult {
        info!("{name} was executed");
        Ok(())
    }
}

impl ActionTrait for JustLogAction {
    fn name(&self) -> &'static str {
        "JustLogAction"
    }
    fn dbg_fmt(&self, _nest: usize, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
    fn try_execute(&mut self) -> ReusableBoxFutureResult {
        self.base.reusable_future_pool.next(JustLogAction::execute_impl(self.name.clone()))
    }
}

fn acc_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("ACC_design".into(), DesignConfig::default());

    design.register_event("cyclic_evt".into())?; // Register a timer event
    design.register_event("trigger_acc".into())?;
    design.register_event("trigger_s2m".into())?;

    design.add_program("acc", move |design, builder| {
        builder
            .with_start_action(JustLogAction::new("StartACC"))
            .with_run_action(
                SequenceBuilder::new()
                    .with_step(SyncBuilder::from_design("trigger_acc", design))
                    .with_step(JustLogAction::new("RunACC"))
                    .with_step(TriggerBuilder::from_design("trigger_s2m", design))
                    .build(),
            )
            .with_stop_action(JustLogAction::new("StopACC"), std::time::Duration::from_secs(10));

        Ok(())
    });

    Ok(design)
}

fn m2s_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("M2S_design".into(), DesignConfig::default());

    design.register_event("cyclic_evt".into())?; // Register a timer event
    design.register_event("trigger_acc".into())?;
    design.register_event("trigger_s2m".into())?;

    design.add_program("m2s", move |design, builder| {
        builder
            .with_start_action(JustLogAction::new("StartM2S"))
            .with_run_action(
                SequenceBuilder::new()
                    .with_step(SyncBuilder::from_design("cyclic_evt", design))
                    .with_step(JustLogAction::new("RunM2S"))
                    .with_step(TriggerBuilder::from_design("trigger_acc", design))
                    .build(),
            )
            .with_stop_action(JustLogAction::new("StopM2S"), Duration::from_secs(10));

        Ok(())
    });

    Ok(design)
}

fn s2m_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("S2M_design".into(), DesignConfig::default());

    design.register_event("trigger_s2m".into())?;

    design.add_program("s2m", move |design, builder| {
        builder
            .with_start_action(JustLogAction::new("StartS2M"))
            .with_run_action(
                SequenceBuilder::new()
                    .with_step(SyncBuilder::from_design("trigger_s2m", design))
                    .with_step(JustLogAction::new("RunS2M"))
                    .build(),
            )
            .with_stop_action(JustLogAction::new("StopS2M"), Duration::from_secs(10));

        Ok(())
    });

    Ok(design)
}

pub struct ProgramDemo;

impl Scenario for ProgramDemo {
    fn name(&self) -> &str {
        "demo"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut runtime = Runtime::from_json(input)?.build();
        let logic = DemoLogic::new(input);

        // Build Orchestration
        let mut orch = Orchestration::new()
            .add_design(acc_design().expect("Failed to create design"))
            .add_design(s2m_design().expect("Failed to create design"))
            .add_design(m2s_design().expect("Failed to create design"))
            .design_done();

        // Deployment part - specify event details
        let mut deployment = orch.get_deployment_mut();

        deployment
            .bind_events_as_local(&["trigger_acc".into()])
            .expect("Failed to bind trigger_acc");

        deployment
            .bind_events_as_local(&["trigger_s2m".into()])
            .expect("Failed to bind trigger_s2m");

        deployment
            .bind_events_as_timer(&["cyclic_evt".into()], Duration::from_millis(logic.cycle_duration_ms))
            .expect("Failed to bind cyclic_evt as timer");

        // Create programs
        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut program_acc = program_manager.get_program("acc").expect("Failed to get acc program");
        let mut program_s2m = program_manager.get_program("s2m").expect("Failed to get s2m program");
        let mut program_m2s = program_manager.get_program("m2s").expect("Failed to get m2s program");

        // Put programs into runtime and run them
        runtime.block_on(async move {
            let h1 = kyron::spawn(async move {
                let _ = program_acc.run().await;
            });

            let h2 = kyron::spawn(async move {
                let _ = program_s2m.run().await;
            });

            let h3 = kyron::spawn(async move {
                let _ = program_m2s.run().await;
            });

            let _ = h1.await;
            let _ = h2.await;
            let _ = h3.await;

            info!("Programs finished running");
        });

        info!("Exit.");

        Ok(())
    }
}
