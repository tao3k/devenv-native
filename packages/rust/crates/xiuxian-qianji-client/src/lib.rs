//! Xiuxian Qianji client CLI surfaces for downstream project operations.

mod error;
mod flowhub;

pub use error::QianjiClientError;
pub use flowhub::{
    FlowhubAction, FlowhubCliOutput, FlowhubGeneratedFile, FlowhubScenarioRegistry,
    FlowhubScenarioRegistrySourcePair, FlowhubScenarioRegistryValidation, FlowhubSourcePairSummary,
    load_flowhub_scenario_registry, run_xiuxian_qianji_client_cli,
    run_xiuxian_qianji_client_cli_with_args,
};
