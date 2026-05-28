#[cfg(feature = "wendao-integration")]
use super::render_follow_up_query_section;
use super::{MarkdownDiagnostic, render_validation_failed, render_validation_pass};

#[test]
fn render_validation_pass_uses_shared_header() {
    let rendered =
        render_validation_pass(&["Plan: demo".to_string(), "Location: /tmp/demo".to_string()]);

    assert_eq!(
        rendered,
        "# Validation Passed\n\nPlan: demo\nLocation: /tmp/demo"
    );
}

#[test]
fn render_validation_failed_uses_shared_shape() {
    let rendered = render_validation_failed(
        &["Location: /tmp/demo".to_string()],
        &[MarkdownDiagnostic {
            title: "Example failure",
            location: "/tmp/demo".into(),
            problem: "something is wrong",
            why_it_blocks: "the contract is incomplete",
            fix: "repair the missing surface",
        }],
    );

    assert_eq!(
        rendered,
        "# Validation Failed\n\nLocation: /tmp/demo\n\n## Example failure\nLocation: /tmp/demo\nProblem: something is wrong\nWhy it blocks: the contract is incomplete\nFix: repair the missing surface"
    );
}

#[cfg(feature = "wendao-integration")]
#[test]
fn render_follow_up_query_section_uses_shared_shape() {
    let rendered = render_follow_up_query_section(
        &["blueprint".to_string(), "plan".to_string()],
        "select path, skeleton from markdown order by path",
    );

    assert_eq!(
        rendered,
        "## Follow-up Query\nSurfaces: blueprint, plan\nSQL:\n```sql\nselect path, skeleton from markdown order by path\n```"
    );
}
