//! WorkflowPlan API facade.
//!
//! `api` owns exported DTOs; `validation`, `emission`, and `render` own behavior.

mod api;
mod emission;
mod render;
mod validation;
pub use api::{
    WorkflowPlan, WorkflowPlanDiagnostic, WorkflowPlanDiagnosticSeverity, WorkflowPlanEdge,
    WorkflowPlanEmitError, WorkflowPlanTask, WorkflowPlanValidationReport,
};
#[path = "facade.rs"]
mod facade;

pub use facade::{
    emit_workflow_plan_bpmn, render_workflow_plan_validation_report,
    render_workflow_plan_validation_report_json, validate_workflow_plan,
};
