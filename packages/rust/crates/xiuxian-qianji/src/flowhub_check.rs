//! Flowhub check seam. Start in `api`.

#[path = "flowhub_check/api.rs"]
mod api;
#[path = "flowhub_check/contract.rs"]
mod contract;
#[path = "flowhub_check/filesystem.rs"]
mod filesystem;
#[path = "flowhub_check/mermaid.rs"]
mod mermaid;
#[path = "flowhub_check/traversal.rs"]
mod traversal;

pub use api::{
    FlowhubCheckReport, FlowhubDiagnostic, check_flowhub, render_flowhub_check_markdown,
};
