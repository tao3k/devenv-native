//! Workdir check seam. Start in `api`.

#[path = "workdir_check/api.rs"]
mod api;
#[path = "workdir_check/filesystem.rs"]
mod filesystem;
#[path = "workdir_check/flowchart.rs"]
mod flowchart;
#[path = "workdir_check/render.rs"]
mod render;
#[path = "workdir_check/runtime.rs"]
mod runtime;

pub use api::{
    WorkdirCheckReport, WorkdirDiagnostic, WorkdirMarkdownSurface, check_workdir,
    render_workdir_check_markdown,
};
