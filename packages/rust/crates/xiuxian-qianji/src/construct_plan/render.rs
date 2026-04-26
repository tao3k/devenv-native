use super::api::WorkflowPlanValidationReport;

/// Render a `WorkflowPlan` validation report as markdown.
#[must_use]
pub(crate) fn render_workflow_plan_validation_report(
    report: &WorkflowPlanValidationReport,
) -> String {
    let mut lines = vec![
        "# Qianji Construct WorkflowPlan Validation".to_string(),
        String::new(),
        format!("Status: {}", if report.ok { "passed" } else { "failed" }),
    ];
    if report.diagnostics.is_empty() {
        lines.push(String::new());
        lines.push("No blocking construct-plan issues found.".to_string());
        return lines.join("\n");
    }

    lines.push(String::new());
    lines.push("## Diagnostics".to_string());
    lines.push(String::new());
    for diagnostic in &report.diagnostics {
        lines.push(format!(
            "- `{}` at `{}`: {}",
            diagnostic.code, diagnostic.path, diagnostic.message
        ));
        lines.push(format!("  Repair: {}", diagnostic.repair));
    }
    lines.join("\n")
}

/// Render a `WorkflowPlan` validation report as pretty JSON.
///
/// # Errors
///
/// Returns an error if the report cannot be serialized.
pub(crate) fn render_workflow_plan_validation_report_json(
    report: &WorkflowPlanValidationReport,
) -> serde_json::Result<String> {
    serde_json::to_string_pretty(report)
}
