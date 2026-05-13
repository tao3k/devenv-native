use crate::markdown::{
    MarkdownDiagnostic, render_follow_up_query_section, render_validation_failed,
    render_validation_pass,
};
use crate::workdir::build_workdir_check_follow_up_query;

use super::model::{WorkdirCheckReport, WorkdirMarkdownSurface};

pub(super) fn render_workdir_check_markdown_impl(report: &WorkdirCheckReport) -> String {
    if report.is_valid() {
        return render_validation_pass(&[
            format!("Plan: {}", report.plan_name),
            format!("Location: {}", report.workdir.display()),
        ]);
    }

    let diagnostics = report
        .diagnostics
        .iter()
        .map(|diagnostic| MarkdownDiagnostic {
            title: diagnostic.title.as_str(),
            location: diagnostic.location.display().to_string().into(),
            problem: diagnostic.problem.as_str(),
            why_it_blocks: diagnostic.why_it_blocks.as_str(),
            fix: diagnostic.fix.as_str(),
        })
        .collect::<Vec<_>>();

    let mut rendered = render_validation_failed(&[], &diagnostics);
    if let Some(follow_up_query) = build_workdir_check_follow_up_query(report) {
        let surface_names = follow_up_query
            .surfaces
            .iter()
            .map(|surface| surface.as_str())
            .collect::<Vec<_>>()
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        rendered.push_str("\n\n");
        rendered.push_str(&render_follow_up_query_section(
            &surface_names,
            &follow_up_query.query_text,
        ));
    }

    rendered
}

pub(super) fn follow_up_surfaces_for_requirement(requirement: &str) -> Vec<WorkdirMarkdownSurface> {
    let mut surfaces = Vec::new();
    if requirement.starts_with("blueprint") {
        surfaces.push(WorkdirMarkdownSurface::Blueprint);
    }
    if requirement.starts_with("plan") {
        surfaces.push(WorkdirMarkdownSurface::Plan);
    }
    if requirement.starts_with("semantic") {
        surfaces.push(WorkdirMarkdownSurface::Semantic);
    }
    surfaces
}

pub(super) fn follow_up_surfaces_for_flowchart(entries: &[String]) -> Vec<WorkdirMarkdownSurface> {
    let mut surfaces = entries
        .iter()
        .filter_map(|entry| WorkdirMarkdownSurface::from_top_level_name(entry))
        .collect::<Vec<_>>();
    if surfaces.is_empty() {
        surfaces.push(WorkdirMarkdownSurface::Blueprint);
        surfaces.push(WorkdirMarkdownSurface::Plan);
    }
    surfaces
}

pub(super) fn render_label_list(values: &[String]) -> String {
    if values.is_empty() {
        return "`none`".to_string();
    }

    values
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}
