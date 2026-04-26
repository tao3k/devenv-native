use std::collections::HashSet;

use crate::construct_cards::find_construct_card;

use super::api::{
    WorkflowPlan, WorkflowPlanDiagnostic, WorkflowPlanValidationReport, diagnostic,
    is_variable_path,
};

/// Validate a `WorkflowPlan` against the current construct-card contract subset.
#[must_use]
pub(crate) fn validate_workflow_plan(plan: &WorkflowPlan) -> WorkflowPlanValidationReport {
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
        validate_task_inputs(index, task.inputs.iter().map(String::as_str), diagnostics);
    }
}

fn validate_task_inputs<'a>(
    task_index: usize,
    inputs: impl Iterator<Item = &'a str>,
    diagnostics: &mut Vec<WorkflowPlanDiagnostic>,
) {
    for (input_index, input) in inputs.enumerate() {
        if !is_variable_path(input) {
            diagnostics.push(diagnostic(
                "construct_plan.invalid_input_variable",
                format!("tasks[{task_index}].inputs[{input_index}]"),
                format!("invalid input variable `{input}`"),
                "Use dotted identifier paths such as prompt or context.summary.",
            ));
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
            validate_conditional_edge(
                index,
                condition,
                selected_constructs,
                declared_outputs,
                diagnostics,
            );
        }
    }
}

fn validate_conditional_edge(
    edge_index: usize,
    condition: &str,
    selected_constructs: &HashSet<&str>,
    declared_outputs: &HashSet<&str>,
    diagnostics: &mut Vec<WorkflowPlanDiagnostic>,
) {
    if !selected_constructs.contains("gateway.exclusive.bounded") {
        diagnostics.push(diagnostic(
            "construct_plan.gateway_construct_not_selected",
            format!("edges[{edge_index}].condition"),
            "conditional edge requires `gateway.exclusive.bounded`",
            "Add `gateway.exclusive.bounded` to `constructs` or remove the condition.",
        ));
    }
    validate_condition(condition, edge_index, declared_outputs, diagnostics);
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

fn is_valid_source(source: &str, task_ids: &HashSet<&str>) -> bool {
    source == "start" || task_ids.contains(source)
}

fn is_valid_target(target: &str, task_ids: &HashSet<&str>) -> bool {
    target == "end" || task_ids.contains(target)
}
