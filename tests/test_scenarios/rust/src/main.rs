mod internals;
mod scenarios;

use test_scenarios_rust::cli::{create_tracing_subscriber, run_cli_app};
use test_scenarios_rust::test_context::TestContext;

use crate::scenarios::root_scenario_group;

fn main() -> Result<(), String> {
    let subscriber = create_tracing_subscriber();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed!");

    let raw_arguments: Vec<String> = std::env::args().collect();

    // Root group.
    let root_group = root_scenario_group();

    // Run.
    let test_context = TestContext::new(root_group);
    run_cli_app(&raw_arguments, &test_context)
}
