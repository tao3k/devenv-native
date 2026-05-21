//! Workdir check seam. Start in `api`.

#[path = "api.rs"]
mod api;
#[path = "filesystem.rs"]
mod filesystem;
#[path = "flowchart.rs"]
mod flowchart;
#[path = "model.rs"]
mod model;
#[path = "render.rs"]
mod render;
#[path = "runtime.rs"]
mod runtime;

pub use api::{check_workdir, render_workdir_check_markdown};
pub use model::{WorkdirCheckReport, WorkdirDiagnostic, WorkdirMarkdownSurface};
