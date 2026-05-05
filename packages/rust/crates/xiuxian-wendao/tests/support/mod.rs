//! Test support module for xiuxian-wendao.
//!
//! Provides wendao-specific scenario runners for the unified test framework.

pub mod runners;
pub mod scenario;

pub use runners::{GraphRunner, PageIndexRunner, SearchRunner, SemanticCheckRunner};
pub use scenario::{
    Scenario, ScenarioFramework, ScenarioRunner, ScenarioSnapshotPolicy, find_first_doc_name,
};
