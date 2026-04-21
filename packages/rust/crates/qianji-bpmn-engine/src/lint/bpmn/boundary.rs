use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn boundary_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    match detail {
        "cancel_boundary_requires_transaction_shell" => {
            cancel_boundary_requires_transaction_shell_issue(process_id, node_id, detail)
        }
        "multiple_transaction_cancel_boundaries" => {
            multiple_transaction_cancel_boundaries_issue(process_id, node_id, detail)
        }
        "error_boundary_requires_transaction_shell" => {
            error_boundary_requires_transaction_shell_issue(process_id, node_id, detail)
        }
        _ => generic_boundary_configuration_issue(process_id, node_id, detail),
    }
}

fn cancel_boundary_requires_transaction_shell_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_boundary_configuration",
        "Cancel boundary must attach to a transaction shell",
        format!(
            "Process '{process_id}' boundary event '{node_id}' uses `<cancelEventDefinition>` without attaching it to a bounded transaction shell."
        ),
        "The bounded engine supports one interrupting cancel boundary path only when the boundary event is attached to one bounded `<transaction>` shell and matches the transaction shell's nested cancel end.",
        vec![
            "Attach this cancel boundary to one `<bpmn:transaction>` node, not to a task, embedded subprocess, or call activity.".to_string(),
            "Keep `cancelActivity=\"true\"` and pair the boundary with exactly one nested transaction end event that carries `<bpmn:cancelEventDefinition>`.".to_string(),
        ],
        format!(
            "Rewrite boundary event '{node_id}' in process '{process_id}' so `<cancelEventDefinition>` is used only as one interrupting boundary event attached to one bounded `<bpmn:transaction>` shell, paired with exactly one nested cancel end inside that same transaction. Preserve workflow intent, but do not leave the cancel boundary attached to a task or non-transaction subprocess."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn multiple_transaction_cancel_boundaries_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_boundary_configuration",
        "Transaction owner exposes more than one cancel boundary",
        format!(
            "Process '{process_id}' boundary event '{node_id}' adds a second `<cancelEventDefinition>` boundary to the same bounded transaction owner."
        ),
        "The bounded engine allows one transaction owner to expose one interrupting cancel boundary plus one or more interrupting error boundaries, but it still permits only one cancel boundary on that same transaction shell.",
        vec![
            "Keep exactly one interrupting cancel boundary attached to this `<bpmn:transaction>` node.".to_string(),
            "If the second branch is really an error path, rewrite it as `<bpmn:errorEventDefinition>` and pair it with the transaction shell's nested error end.".to_string(),
        ],
        format!(
            "Repair process '{process_id}' so boundary event '{node_id}' no longer creates a second transaction cancel boundary. Keep one interrupting `<bpmn:cancelEventDefinition>` boundary on that transaction owner, and convert any extra recovery branch into one or more interrupting `<bpmn:errorEventDefinition>` boundaries only if the nested transaction end throws matching errors."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn error_boundary_requires_transaction_shell_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_boundary_configuration",
        "Error boundary must attach to a transaction shell",
        format!(
            "Process '{process_id}' boundary event '{node_id}' uses `<errorEventDefinition>` without attaching it to a bounded transaction shell."
        ),
        "The bounded engine supports one or more interrupting error boundary paths only when those boundary events are attached to one bounded `<transaction>` shell and match that transaction shell's nested error end.",
        vec![
            "Attach this error boundary to one `<bpmn:transaction>` node, not to a task, embedded subprocess, or call activity.".to_string(),
            "Keep `cancelActivity=\"true\"` and pair the boundary with exactly one nested transaction end event that carries `<bpmn:errorEventDefinition>`, using a matching `errorRef` or omitting `errorRef` on the boundary as a catch-all.".to_string(),
        ],
        format!(
            "Rewrite boundary event '{node_id}' in process '{process_id}' so `<errorEventDefinition>` is used only as one interrupting boundary event attached to one bounded `<bpmn:transaction>` shell, paired with exactly one nested error end inside that same transaction. Preserve workflow intent, but do not leave the error boundary attached to a task or non-transaction subprocess."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn generic_boundary_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_boundary_configuration",
        "Boundary event configuration exceeds the bounded slice",
        format!(
            "Process '{process_id}' boundary event '{node_id}' uses unsupported configuration '{detail}'."
        ),
        "The current engine supports only three interrupting boundary ownership shapes: one timer boundary attached to one host-blocking task, one cancel boundary attached to one bounded transaction shell, or one or more error boundaries attached to one bounded transaction shell.",
        vec![
            "Keep the timeout, escalation, or transaction-cancel intent, but rewrite the boundary to one supported interrupting shape.".to_string(),
            "Use a timer boundary on one serviceTask, userTask, manualTask, or businessRuleTask, or use one cancel boundary plus one or more error boundaries on one bounded `<transaction>` shell.".to_string(),
        ],
        format!(
            "Rewrite boundary event '{node_id}' in process '{process_id}' so it fits the bounded slice: either one interrupting timer `boundaryEvent` attached to one serviceTask, userTask, manualTask, or businessRuleTask with `cancelActivity=\"true\"` and exactly one timer expression, one interrupting cancel `boundaryEvent` attached to one bounded `<bpmn:transaction>` shell with a matching nested cancel end, or one or more interrupting error `boundaryEvent` nodes attached to one bounded `<bpmn:transaction>` shell with a matching nested error end whose optional `errorRef` either matches the thrown error or stays omitted as a catch-all. Preserve workflow intent, but remove unsupported configuration '{detail}'."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}
