mod check;
mod detect;
mod load;
mod parse;
mod query;
mod show;
mod validate;

pub use check::{
    WorkdirCheckReport, WorkdirDiagnostic, WorkdirMarkdownSurface, check_workdir,
    render_workdir_check_markdown,
};
pub use detect::looks_like_workdir_dir;
pub use load::load_workdir_manifest;
pub use parse::parse_workdir_manifest;
pub use query::{
    WorkdirCheckFollowUpQuery, build_workdir_check_follow_up_query,
    query_workdir_check_follow_up_payload, query_workdir_markdown_payload,
};
pub use show::{
    WorkdirShow, WorkdirVisibleSurface, WorkdirVisibleSurfaceKind, render_workdir_show,
    show_workdir,
};
pub(crate) use validate::validate_workdir_manifest;
