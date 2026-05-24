//! Flowhub check seam. Start in `api`.

#[path = "api.rs"]
mod api;
#[path = "contract.rs"]
mod contract;
#[path = "filesystem.rs"]
mod filesystem;
#[path = "mermaid.rs"]
mod mermaid;
#[path = "model.rs"]
mod model;
#[path = "source_pair.rs"]
mod source_pair;
#[path = "traversal.rs"]
mod traversal;

pub use api::{check_flowhub, render_flowhub_check_markdown};
pub use model::{FlowhubCheckReport, FlowhubDiagnostic};
