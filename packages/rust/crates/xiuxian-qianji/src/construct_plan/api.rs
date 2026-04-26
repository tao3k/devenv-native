use serde::{Deserialize, Serialize};

/// Minimal pre-emission workflow plan produced after construct-card selection.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct WorkflowPlan {
    /// Schema version. The current validator accepts version 1.
    pub version: u32,
    /// Human-readable plan name.
    pub name: String,
    /// Selected construct-card ids.
    #[serde(default)]
    pub constructs: Vec<String>,
    /// Host or decision tasks in the plan.
    #[serde(default)]
    pub tasks: Vec<WorkflowPlanTask>,
    /// Directed edges between `start`, task ids, and `end`.
    #[serde(default)]
    pub edges: Vec<WorkflowPlanEdge>,
}

/// One task in a `WorkflowPlan`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct WorkflowPlanTask {
    /// Stable task id.
    pub id: String,
    /// Construct-card id used by this task.
    pub construct: String,
    /// Input variable names consumed by this task.
    #[serde(default)]
    pub inputs: Vec<String>,
    /// Output variable names produced by this task.
    #[serde(default)]
    pub outputs: Vec<String>,
}

/// One directed edge in a `WorkflowPlan`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct WorkflowPlanEdge {
    /// Source node id: `start` or a task id.
    pub from: String,
    /// Target node id: a task id or `end`.
    pub to: String,
    /// Optional qianji bounded condition expression.
    #[serde(default)]
    pub condition: Option<String>,
    /// Whether this is the default edge from a gateway-like split.
    #[serde(default)]
    pub default: bool,
}

/// Static validation diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowPlanDiagnosticSeverity {
    /// Blocks lowering or execution.
    Error,
}

/// One static `WorkflowPlan` validation diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowPlanDiagnostic {
    /// Stable diagnostic code.
    pub code: &'static str,
    /// Severity level.
    pub severity: WorkflowPlanDiagnosticSeverity,
    /// JSON-ish location in the `WorkflowPlan`.
    pub path: String,
    /// Human-readable diagnostic message.
    pub message: String,
    /// Repair guidance intended for LLM consumers.
    pub repair: String,
}

/// `WorkflowPlan` validation report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowPlanValidationReport {
    /// Whether validation produced no blocking diagnostics.
    pub ok: bool,
    /// Diagnostics found during validation.
    pub diagnostics: Vec<WorkflowPlanDiagnostic>,
}

/// Error returned when a `WorkflowPlan` cannot be emitted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowPlanEmitError {
    /// Validation report that blocked emission.
    pub validation: WorkflowPlanValidationReport,
}

pub(super) fn diagnostic(
    code: &'static str,
    path: impl Into<String>,
    message: impl Into<String>,
    repair: impl Into<String>,
) -> WorkflowPlanDiagnostic {
    WorkflowPlanDiagnostic {
        code,
        severity: WorkflowPlanDiagnosticSeverity::Error,
        path: path.into(),
        message: message.into(),
        repair: repair.into(),
    }
}

pub(super) fn is_variable_path(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed != value {
        return false;
    }
    trimmed.split('.').all(is_identifier_segment)
}

pub(super) fn escape_xml_attr(value: &str) -> String {
    escape_xml_text(value).replace('"', "&quot;")
}

pub(super) fn escape_xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn is_identifier_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}
