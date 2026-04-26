//! Static WorkflowPlan validation for qianji construct-card consumers.

use std::{
    collections::HashSet,
    fmt::{self, Write as _},
};

use serde::{Deserialize, Serialize};

use crate::construct_cards::find_construct_card;

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

/// Validate a `WorkflowPlan` against the current construct-card contract subset.
#[must_use]
pub fn validate_workflow_plan(plan: &WorkflowPlan) -> WorkflowPlanValidationReport {
    let mut diagnostics = Vec::new();
    validate_plan_metadata(plan, &mut diagnostics);

    let selected_constructs = plan
        .constructs
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    validate_selected_constructs(plan, &selected_constructs, &mut diagnostics);

    let task_ids = collect_task_ids(plan, &mut diagnostics);
    let declared_outputs = collect_declared_outputs(plan, &mut diagnostics);
    validate_tasks(plan, &selected_constructs, &mut diagnostics);
    validate_edges(
        plan,
        &selected_constructs,
        &task_ids,
        &declared_outputs,
        &mut diagnostics,
    );

    WorkflowPlanValidationReport {
        ok: diagnostics.is_empty(),
        diagnostics,
    }
}

/// Render a `WorkflowPlan` validation report as markdown.
#[must_use]
pub fn render_workflow_plan_validation_report(report: &WorkflowPlanValidationReport) -> String {
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
pub fn render_workflow_plan_validation_report_json(
    report: &WorkflowPlanValidationReport,
) -> serde_json::Result<String> {
    serde_json::to_string_pretty(report)
}

/// Emit a validated `WorkflowPlan` as deterministic BPMN XML.
///
/// # Errors
///
/// Returns validation diagnostics when the plan is outside the supported
/// construct subset.
pub fn emit_workflow_plan_bpmn(plan: &WorkflowPlan) -> Result<String, WorkflowPlanEmitError> {
    let validation = validate_workflow_plan(plan);
    if !validation.ok {
        return Err(WorkflowPlanEmitError { validation });
    }

    let process_id = stable_xml_id("Process", &plan.name);
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<definitions xmlns=\"http://www.omg.org/spec/BPMN/20100524/MODEL\"\n");
    xml.push_str("             xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"\n");
    xml.push_str("             xmlns:qianji=\"https://qianji.dev/bpmn/extensions\"\n");
    xml.push_str("             id=\"Definitions_1\"\n");
    xml.push_str("             targetNamespace=\"https://qianji.dev\">\n");
    push_xml(
        &mut xml,
        format_args!(
            "  <process id=\"{}\" name=\"{}\" isExecutable=\"true\">\n",
            process_id,
            escape_xml_attr(&plan.name)
        ),
    );
    xml.push_str("    <startEvent id=\"Start_1\" name=\"Start\"/>\n");

    for task in &plan.tasks {
        push_task_xml(&mut xml, task);
    }
    let gateway_sources = conditional_gateway_sources(plan);
    for gateway in &gateway_sources {
        push_xml(
            &mut xml,
            format_args!(
                "    <exclusiveGateway id=\"{}\" name=\"Route {}\"{} />\n",
                gateway_id(gateway),
                escape_xml_attr(gateway),
                default_flow_for_source(plan, gateway, gateway_sources.len())
                    .map(|flow_id| format!(" default=\"{flow_id}\""))
                    .unwrap_or_default()
            ),
        );
    }

    xml.push_str("    <endEvent id=\"End_1\" name=\"End\"/>\n");
    for (index, source) in gateway_sources.iter().enumerate() {
        push_sequence_flow_xml(
            &mut xml,
            &flow_id(index),
            &node_ref(source),
            &gateway_id(source),
            None,
        );
    }
    for (index, edge) in plan.edges.iter().enumerate() {
        let flow_index = gateway_sources.len() + index;
        push_edge_xml(&mut xml, &gateway_sources, edge, flow_index);
    }
    xml.push_str("  </process>\n");
    xml.push_str("</definitions>");
    Ok(xml)
}

fn validate_plan_metadata(plan: &WorkflowPlan, diagnostics: &mut Vec<WorkflowPlanDiagnostic>) {
    if plan.version != 1 {
        diagnostics.push(diagnostic(
            "construct_plan.unsupported_version",
            "version",
            format!("unsupported WorkflowPlan version {}", plan.version),
            "Use WorkflowPlan version 1 for the current qianji construct validator.",
        ));
    }
    if plan.name.trim().is_empty() {
        diagnostics.push(diagnostic(
            "construct_plan.empty_name",
            "name",
            "WorkflowPlan name is empty",
            "Set a short non-empty plan name.",
        ));
    }
}

fn validate_selected_constructs(
    plan: &WorkflowPlan,
    selected_constructs: &HashSet<&str>,
    diagnostics: &mut Vec<WorkflowPlanDiagnostic>,
) {
    if selected_constructs.is_empty() {
        diagnostics.push(diagnostic(
            "construct_plan.no_constructs",
            "constructs",
            "WorkflowPlan does not select any construct cards",
            "Run `qianji construct index`, inspect relevant cards, and list selected construct ids.",
        ));
    }
    let mut seen_constructs = HashSet::new();
    for (index, construct) in plan.constructs.iter().enumerate() {
        if !seen_constructs.insert(construct.as_str()) {
            diagnostics.push(diagnostic(
                "construct_plan.duplicate_construct",
                format!("constructs[{index}]"),
                format!("duplicate construct `{construct}`"),
                "Treat `constructs` as a set: list each selected construct id once, even when multiple tasks use it.",
            ));
        }
        if find_construct_card(construct).is_none() {
            diagnostics.push(diagnostic(
                "construct_plan.unknown_construct",
                format!("constructs[{index}]"),
                format!("unknown construct `{construct}`"),
                "Use `qianji construct index` to select a registered construct id.",
            ));
        }
    }
}

fn collect_task_ids<'a>(
    plan: &'a WorkflowPlan,
    diagnostics: &mut Vec<WorkflowPlanDiagnostic>,
) -> HashSet<&'a str> {
    let mut task_ids = HashSet::new();
    for (index, task) in plan.tasks.iter().enumerate() {
        if task.id.trim().is_empty() {
            diagnostics.push(diagnostic(
                "construct_plan.empty_task_id",
                format!("tasks[{index}].id"),
                "task id is empty",
                "Use a stable non-empty task id.",
            ));
            continue;
        }
        if !task_ids.insert(task.id.as_str()) {
            diagnostics.push(diagnostic(
                "construct_plan.duplicate_task_id",
                format!("tasks[{index}].id"),
                format!("duplicate task id `{}`", task.id),
                "Use unique task ids across the WorkflowPlan.",
            ));
        }
    }
    task_ids
}

fn collect_declared_outputs<'a>(
    plan: &'a WorkflowPlan,
    diagnostics: &mut Vec<WorkflowPlanDiagnostic>,
) -> HashSet<&'a str> {
    let mut declared_outputs = HashSet::new();
    for (task_index, task) in plan.tasks.iter().enumerate() {
        for (output_index, output) in task.outputs.iter().enumerate() {
            if !is_variable_path(output) {
                diagnostics.push(diagnostic(
                    "construct_plan.invalid_output_variable",
                    format!("tasks[{task_index}].outputs[{output_index}]"),
                    format!("invalid output variable `{output}`"),
                    "Use dotted identifier paths such as approved or result.ready.",
                ));
                continue;
            }
            declared_outputs.insert(output.as_str());
        }
    }
    declared_outputs
}

fn validate_tasks(
    plan: &WorkflowPlan,
    selected_constructs: &HashSet<&str>,
    diagnostics: &mut Vec<WorkflowPlanDiagnostic>,
) {
    for (index, task) in plan.tasks.iter().enumerate() {
        if find_construct_card(&task.construct).is_none() {
            diagnostics.push(diagnostic(
                "construct_plan.unknown_task_construct",
                format!("tasks[{index}].construct"),
                format!(
                    "task `{}` uses unknown construct `{}`",
                    task.id, task.construct
                ),
                "Use a registered construct id from `qianji construct index`.",
            ));
            continue;
        }
        if !selected_constructs.contains(task.construct.as_str()) {
            diagnostics.push(diagnostic(
                "construct_plan.task_construct_not_selected",
                format!("tasks[{index}].construct"),
                format!(
                    "task `{}` uses construct `{}`, but it is not listed in `constructs`",
                    task.id, task.construct
                ),
                "Add the task construct id to the WorkflowPlan `constructs` array.",
            ));
        }
        if task.construct.starts_with("gateway.") {
            diagnostics.push(diagnostic(
                "construct_plan.gateway_as_task",
                format!("tasks[{index}].construct"),
                format!(
                    "task `{}` uses gateway construct `{}` as a task",
                    task.id, task.construct
                ),
                "Represent gateway routing on edges, not as a task node.",
            ));
        }
        for (input_index, input) in task.inputs.iter().enumerate() {
            if !is_variable_path(input) {
                diagnostics.push(diagnostic(
                    "construct_plan.invalid_input_variable",
                    format!("tasks[{index}].inputs[{input_index}]"),
                    format!("invalid input variable `{input}`"),
                    "Use dotted identifier paths such as prompt or context.summary.",
                ));
            }
        }
    }
}

fn validate_edges(
    plan: &WorkflowPlan,
    selected_constructs: &HashSet<&str>,
    task_ids: &HashSet<&str>,
    declared_outputs: &HashSet<&str>,
    diagnostics: &mut Vec<WorkflowPlanDiagnostic>,
) {
    for (index, edge) in plan.edges.iter().enumerate() {
        if !is_valid_source(&edge.from, task_ids) {
            diagnostics.push(diagnostic(
                "construct_plan.unknown_edge_source",
                format!("edges[{index}].from"),
                format!("edge source `{}` is not `start` or a task id", edge.from),
                "Set edge source to `start` or a declared task id.",
            ));
        }
        if !is_valid_target(&edge.to, task_ids) {
            diagnostics.push(diagnostic(
                "construct_plan.unknown_edge_target",
                format!("edges[{index}].to"),
                format!("edge target `{}` is not `end` or a task id", edge.to),
                "Set edge target to `end` or a declared task id.",
            ));
        }
        if let Some(condition) = edge.condition.as_deref() {
            if !selected_constructs.contains("gateway.exclusive.bounded") {
                diagnostics.push(diagnostic(
                    "construct_plan.gateway_construct_not_selected",
                    format!("edges[{index}].condition"),
                    "conditional edge requires `gateway.exclusive.bounded`",
                    "Add `gateway.exclusive.bounded` to `constructs` or remove the condition.",
                ));
            }
            validate_condition(condition, index, declared_outputs, diagnostics);
        }
    }
}

fn validate_condition(
    condition: &str,
    edge_index: usize,
    declared_outputs: &HashSet<&str>,
    diagnostics: &mut Vec<WorkflowPlanDiagnostic>,
) {
    let trimmed = condition.trim();
    if trimmed.is_empty() {
        diagnostics.push(diagnostic(
            "construct_plan.empty_condition",
            format!("edges[{edge_index}].condition"),
            "condition is empty",
            "Use a plain declared boolean variable, `not variable`, or a numeric comparison.",
        ));
        return;
    }
    if contains_forbidden_condition_syntax(trimmed) {
        diagnostics.push(diagnostic(
            "construct_plan.unsupported_condition",
            format!("edges[{edge_index}].condition"),
            format!("condition `{trimmed}` is outside the bounded qianji subset"),
            "Use a plain boolean variable, `not variable`, or a numeric comparison such as retryCount >= 3.",
        ));
        return;
    }
    let Some(variable) = condition_variable(trimmed) else {
        diagnostics.push(diagnostic(
            "construct_plan.unsupported_condition",
            format!("edges[{edge_index}].condition"),
            format!("condition `{trimmed}` is outside the bounded qianji subset"),
            "Move rich logic into an upstream task that outputs a declared boolean.",
        ));
        return;
    };
    if !declared_outputs.contains(variable) {
        diagnostics.push(diagnostic(
            "construct_plan.undeclared_condition_variable",
            format!("edges[{edge_index}].condition"),
            format!("condition references undeclared output variable `{variable}`"),
            "Declare this variable in an upstream task `outputs` array, or route on an existing declared output.",
        ));
    }
}

fn contains_forbidden_condition_syntax(condition: &str) -> bool {
    ["${", "==", "&&", "||", "\"", "'", "(", ")"]
        .iter()
        .any(|token| condition.contains(token))
}

fn condition_variable(condition: &str) -> Option<&str> {
    if let Some(rest) = condition.strip_prefix("not ") {
        return is_variable_path(rest).then_some(rest);
    }
    for operator in [">=", "<=", ">", "<"] {
        if let Some((left, right)) = condition.split_once(operator) {
            let variable = left.trim();
            let literal = right.trim();
            return (is_variable_path(variable) && literal.parse::<f64>().is_ok())
                .then_some(variable);
        }
    }
    is_variable_path(condition).then_some(condition)
}

fn is_variable_path(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed != value {
        return false;
    }
    trimmed.split('.').all(is_identifier_segment)
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

fn is_valid_source(source: &str, task_ids: &HashSet<&str>) -> bool {
    source == "start" || task_ids.contains(source)
}

fn is_valid_target(target: &str, task_ids: &HashSet<&str>) -> bool {
    target == "end" || task_ids.contains(target)
}

fn push_task_xml(xml: &mut String, task: &WorkflowPlanTask) {
    let element = match task.construct.as_str() {
        "user-task.interaction" => "userTask",
        _ => "serviceTask",
    };
    let implementation = if element == "serviceTask" {
        " implementation=\"${environment.services.runAgent}\""
    } else {
        ""
    };
    push_xml(
        xml,
        format_args!(
            "    <{element} id=\"{}\" name=\"{}\"{implementation}>\n",
            escape_xml_attr(&task.id),
            escape_xml_attr(&task.id)
        ),
    );
    xml.push_str("      <extensionElements>\n");
    xml.push_str("        <qianji:config>\n");
    push_xml(
        xml,
        format_args!(
            "          <qianji:prompt>{}</qianji:prompt>\n",
            escape_xml_text(&format!("Execute WorkflowPlan task {}.", task.id))
        ),
    );
    if !task.inputs.is_empty() {
        push_xml(
            xml,
            format_args!(
                "          <qianji:inputs>{}</qianji:inputs>\n",
                escape_xml_text(&task.inputs.join(","))
            ),
        );
    }
    if !task.outputs.is_empty() {
        push_xml(
            xml,
            format_args!(
                "          <qianji:outputs>{}</qianji:outputs>\n",
                escape_xml_text(&task.outputs.join(","))
            ),
        );
    }
    xml.push_str("        </qianji:config>\n");
    xml.push_str("      </extensionElements>\n");
    push_xml(xml, format_args!("    </{element}>\n"));
}

fn push_edge_xml(
    xml: &mut String,
    gateway_sources: &[&str],
    edge: &WorkflowPlanEdge,
    index: usize,
) {
    let flow_id = flow_id(index);
    let source_ref = if gateway_sources.contains(&edge.from.as_str()) {
        gateway_id(&edge.from)
    } else {
        node_ref(&edge.from)
    };
    let target_ref = node_ref(&edge.to);
    push_sequence_flow_xml(
        xml,
        &flow_id,
        &source_ref,
        &target_ref,
        edge.condition.as_deref(),
    );
}

fn push_sequence_flow_xml(
    xml: &mut String,
    flow_id: &str,
    source_ref: &str,
    target_ref: &str,
    condition: Option<&str>,
) {
    if let Some(condition) = condition {
        push_xml(
            xml,
            format_args!(
                "    <sequenceFlow id=\"{flow_id}\" sourceRef=\"{source_ref}\" targetRef=\"{target_ref}\">\n"
            ),
        );
        push_xml(
            xml,
            format_args!(
                "      <conditionExpression xsi:type=\"tFormalExpression\">{}</conditionExpression>\n",
                escape_xml_text(condition)
            ),
        );
        xml.push_str("    </sequenceFlow>\n");
    } else {
        push_xml(
            xml,
            format_args!(
                "    <sequenceFlow id=\"{flow_id}\" sourceRef=\"{source_ref}\" targetRef=\"{target_ref}\"/>\n"
            ),
        );
    }
}

fn push_xml(xml: &mut String, args: fmt::Arguments<'_>) {
    let _ = xml.write_fmt(args);
}

fn conditional_gateway_sources(plan: &WorkflowPlan) -> Vec<&str> {
    let mut sources = Vec::new();
    for edge in &plan.edges {
        if edge.condition.is_some() && !sources.contains(&edge.from.as_str()) {
            sources.push(edge.from.as_str());
        }
    }
    sources
}

fn default_flow_for_source(
    plan: &WorkflowPlan,
    source: &str,
    gateway_source_count: usize,
) -> Option<String> {
    plan.edges
        .iter()
        .position(|edge| edge.from == source && edge.default)
        .map(|edge_index| flow_id(gateway_source_count + edge_index))
}

fn flow_id(index: usize) -> String {
    format!("Flow_{}", index + 1)
}

fn gateway_id(source: &str) -> String {
    stable_xml_id("Gateway", source)
}

fn node_ref(node: &str) -> String {
    match node {
        "start" => "Start_1".to_string(),
        "end" => "End_1".to_string(),
        other => other.to_string(),
    }
}

fn stable_xml_id(prefix: &str, value: &str) -> String {
    let mut output = String::with_capacity(prefix.len() + value.len() + 1);
    output.push_str(prefix);
    output.push('_');
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            output.push(ch);
        } else {
            output.push('_');
        }
    }
    if output.ends_with('_') {
        output.push('1');
    }
    output
}

fn escape_xml_attr(value: &str) -> String {
    escape_xml_text(value).replace('"', "&quot;")
}

fn escape_xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn diagnostic(
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
