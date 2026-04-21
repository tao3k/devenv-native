//! Shared markdown renderers for `qianji` show/check surfaces.

mod check;
mod show;

pub(crate) use check::{
    MarkdownDiagnostic, render_follow_up_query_section, render_validation_failed,
    render_validation_pass,
};
pub(crate) use show::{MarkdownShowSection, render_show_surface};
