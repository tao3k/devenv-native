//! Facade surface for `xiuxian-qianji`.

use super::{
    WorkflowPlan, WorkflowPlanEmitError, WorkflowPlanValidationReport, emission, render, validation,
};

/// Validate a `WorkflowPlan` against the current construct-card contract subset.
#[must_use]
pub fn validate_workflow_plan(plan: &WorkflowPlan) -> WorkflowPlanValidationReport {
    validation::validate_workflow_plan(plan)
}

/// Render a `WorkflowPlan` validation report as markdown.
#[must_use]
pub fn render_workflow_plan_validation_report(report: &WorkflowPlanValidationReport) -> String {
    render::render_workflow_plan_validation_report(report)
}

/// Render a `WorkflowPlan` validation report as pretty JSON.
///
/// # Errors
///
/// Returns an error if the report cannot be serialized.
pub fn render_workflow_plan_validation_report_json(
    report: &WorkflowPlanValidationReport,
) -> serde_json::Result<String> {
    render::render_workflow_plan_validation_report_json(report)
}

/// Emit a validated `WorkflowPlan` as deterministic BPMN XML.
///
/// # Errors
///
/// Returns validation diagnostics when the plan is outside the supported
/// construct subset.
pub fn emit_workflow_plan_bpmn(plan: &WorkflowPlan) -> Result<String, WorkflowPlanEmitError> {
    emission::emit_workflow_plan_bpmn(plan)
}
