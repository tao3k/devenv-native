//! High-level laboratory API for running Qianji workflows end-to-end.

mod manifest;
#[path = "../bootcamp_model.rs"]
mod model;
mod runtime;
#[path = "../bootcamp_workflow.rs"]
mod workflow;

pub use model::{BootcampRunOptions, BootcampVfsMount, WorkflowReport};
pub use workflow::{
    run_scenario, run_workflow, run_workflow_from_manifest_toml, run_workflow_with_mounts,
};
