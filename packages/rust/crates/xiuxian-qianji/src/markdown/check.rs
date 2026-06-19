use std::borrow::Cow;

use crate::template_catalog::EmbeddedTemplateCatalog;
use serde_json::json;

const VALIDATION_PASS_TEMPLATE_NAME: &str = "qianji_validation_pass.md.j2";
const VALIDATION_PASS_TEMPLATE_SOURCE: &str =
    include_str!("../../resources/templates/control_plane/qianji_validation_pass.md.j2");

const VALIDATION_FAILED_TEMPLATE_NAME: &str = "qianji_validation_failed.md.j2";
const VALIDATION_FAILED_TEMPLATE_SOURCE: &str =
    include_str!("../../resources/templates/control_plane/qianji_validation_failed.md.j2");

const FOLLOW_UP_QUERY_TEMPLATE_NAME: &str = "qianji_follow_up_query.md.j2";
const FOLLOW_UP_QUERY_TEMPLATE_SOURCE: &str =
    include_str!("../../resources/templates/control_plane/qianji_follow_up_query.md.j2");

static CHECK_TEMPLATE_CATALOG: EmbeddedTemplateCatalog = EmbeddedTemplateCatalog::new(
    "Qianji validation markdown renderer",
    &[
        (
            VALIDATION_PASS_TEMPLATE_NAME,
            VALIDATION_PASS_TEMPLATE_SOURCE,
        ),
        (
            VALIDATION_FAILED_TEMPLATE_NAME,
            VALIDATION_FAILED_TEMPLATE_SOURCE,
        ),
        (
            FOLLOW_UP_QUERY_TEMPLATE_NAME,
            FOLLOW_UP_QUERY_TEMPLATE_SOURCE,
        ),
    ],
);

/// One diagnostic section rendered in the shared markdown validation surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownDiagnostic<'a> {
    /// Short diagnostic title.
    pub(crate) title: &'a str,
    /// On-disk location of the failing surface.
    pub(crate) location: Cow<'a, str>,
    /// Concrete failing condition.
    pub(crate) problem: &'a str,
    /// Why the issue blocks progress.
    pub(crate) why_it_blocks: &'a str,
    /// Concrete next action.
    pub(crate) fix: &'a str,
}

/// Render the shared success markdown surface.
#[must_use]
pub(crate) fn render_validation_pass(summary_lines: &[String]) -> String {
    match CHECK_TEMPLATE_CATALOG
        .render_lines(
            VALIDATION_PASS_TEMPLATE_NAME,
            json!({
                "summary_block": summary_lines.join("\n"),
            }),
        )
        .map(|lines| lines.join("\n"))
    {
        Ok(rendered) => rendered,
        Err(error) => {
            log::warn!("{error}");
            render_validation_pass_inline(summary_lines)
        }
    }
}

/// Render the shared failure markdown surface.
#[must_use]
pub(crate) fn render_validation_failed(
    header_lines: &[String],
    diagnostics: &[MarkdownDiagnostic<'_>],
) -> String {
    let payload = json!({
        "header_block": header_lines.join("\n"),
        "diagnostics": diagnostics
            .iter()
            .map(|diagnostic| {
                json!({
                    "title": diagnostic.title,
                    "body": render_diagnostic_body(diagnostic),
                })
            })
            .collect::<Vec<_>>(),
    });

    match CHECK_TEMPLATE_CATALOG
        .render_lines(VALIDATION_FAILED_TEMPLATE_NAME, payload)
        .map(|lines| lines.join("\n"))
    {
        Ok(rendered) => rendered,
        Err(error) => {
            log::warn!("{error}");
            render_validation_failed_inline(header_lines, diagnostics)
        }
    }
}

/// Render the shared bounded-repair follow-up SQL section.
#[cfg(feature = "wendao-integration")]
#[must_use]
pub(crate) fn render_follow_up_query_section(surface_names: &[String], query_text: &str) -> String {
    match CHECK_TEMPLATE_CATALOG
        .render_lines(
            FOLLOW_UP_QUERY_TEMPLATE_NAME,
            json!({
                "surfaces": surface_names.join(", "),
                "query_text": query_text,
            }),
        )
        .map(|lines| lines.join("\n"))
    {
        Ok(rendered) => rendered,
        Err(error) => {
            log::warn!("{error}");
            render_follow_up_query_section_inline(surface_names, query_text)
        }
    }
}

fn render_validation_pass_inline(summary_lines: &[String]) -> String {
    let mut lines = vec!["# Validation Passed".to_string()];
    if !summary_lines.is_empty() {
        lines.push(String::new());
        lines.extend(summary_lines.iter().cloned());
    }
    lines.join("\n")
}

fn render_validation_failed_inline(
    header_lines: &[String],
    diagnostics: &[MarkdownDiagnostic<'_>],
) -> String {
    let mut lines = vec!["# Validation Failed".to_string()];
    if !header_lines.is_empty() {
        lines.push(String::new());
        lines.extend(header_lines.iter().cloned());
    }
    for diagnostic in diagnostics {
        lines.push(String::new());
        lines.push(format!("## {}", diagnostic.title));
        lines.extend(render_diagnostic_body_lines(diagnostic));
    }
    lines.join("\n")
}

fn render_diagnostic_body(diagnostic: &MarkdownDiagnostic<'_>) -> String {
    render_diagnostic_body_lines(diagnostic).join("\n")
}

fn render_diagnostic_body_lines(diagnostic: &MarkdownDiagnostic<'_>) -> Vec<String> {
    vec![
        format!("Location: {}", diagnostic.location),
        format!("Problem: {}", diagnostic.problem),
        format!("Why it blocks: {}", diagnostic.why_it_blocks),
        format!("Fix: {}", diagnostic.fix),
    ]
}

#[cfg(feature = "wendao-integration")]
fn render_follow_up_query_section_inline(surface_names: &[String], query_text: &str) -> String {
    format!(
        "## Follow-up Query\nSurfaces: {}\nSQL:\n```sql\n{}\n```",
        surface_names.join(", "),
        query_text
    )
}

#[cfg(test)]
#[path = "../../tests/unit/markdown/check.rs"]
mod tests;
