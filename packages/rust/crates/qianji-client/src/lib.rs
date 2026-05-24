//! Qianji client CLI surfaces for downstream project operations.

mod error;
mod flowhub;

pub use error::QianjiClientError;
pub use flowhub::{
    FlowhubAction, FlowhubCliOutput, FlowhubGeneratedFile, FlowhubSourcePairSummary,
    run_qianji_client_cli, run_qianji_client_cli_with_args,
};

#[cfg(test)]
#[path = "../tests/unit/lib_policy.rs"]
mod rust_project_harness_gate;

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = rust_project_harness_gate::qianji_client_harness_config()
);
