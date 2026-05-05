//! Flowhub check seam. Start in `api`.

#[path = "api.rs"]
mod api;
#[path = "contract.rs"]
mod contract;
#[path = "filesystem.rs"]
mod filesystem;
#[path = "mermaid.rs"]
mod mermaid;
#[path = "traversal.rs"]
mod traversal;

pub use api::{
    FlowhubCheckReport, FlowhubDiagnostic, check_flowhub, render_flowhub_check_markdown,
};
