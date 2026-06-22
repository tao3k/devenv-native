use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn transaction_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    match detail {
        "cancel_end_requires_transaction_shell" => {
            cancel_end_requires_transaction_shell_issue(process_id, node_id, detail)
        }
        "error_end_requires_transaction_shell" => {
            error_end_requires_transaction_shell_issue(process_id, node_id, detail)
        }
        "multiple_transaction_cancel_end_events" => {
            multiple_transaction_cancel_end_issue(process_id, node_id, detail)
        }
        "transaction_cancel_missing_boundary" => {
            transaction_cancel_missing_boundary_issue(process_id, node_id, detail)
        }
        "transaction_error_missing_boundary" => {
            transaction_error_missing_boundary_issue(process_id, node_id, detail)
        }
        _ => generic_transaction_configuration_issue(process_id, node_id, detail),
    }
}

fn error_end_requires_transaction_shell_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::from_parts(
        "bpmn.unsupported_transaction_configuration",
        "Error end event must live inside a transaction shell",
        format!(
            "Process '{process_id}' end event '{node_id}' uses `<errorEventDefinition>` outside one bounded transaction shell."
        ),
        "The bounded engine supports a transaction error end only as part of one transaction-error path: the end event must live inside one nested `<bpmn:transaction>` shell and be paired with at least one parent interrupting error boundary attached to that same transaction node.",
        vec![
            "If this is not a real BPMN transaction error path, replace `<bpmn:errorEventDefinition>` with a regular `<bpmn:endEvent>`.".to_string(),
            "If it is a real transaction error path, move the error end inside one `<bpmn:transaction>` body and add the matching parent error boundary.".to_string(),
        ],
        format!(
            "Repair process '{process_id}' so end event '{node_id}' no longer uses `<bpmn:errorEventDefinition>` outside one bounded transaction shell. Either replace it with a regular end event, or move it inside one `<bpmn:transaction>` body and add the matching parent interrupting error boundary attached to that transaction."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn cancel_end_requires_transaction_shell_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::from_parts(
        "bpmn.unsupported_transaction_configuration",
        "Cancel end event must live inside a transaction shell",
        format!(
            "Process '{process_id}' end event '{node_id}' uses `<cancelEventDefinition>` outside one bounded transaction shell."
        ),
        "The bounded engine supports a cancel end only as part of one transaction-cancel path: the end event must live inside one nested `<bpmn:transaction>` shell and be paired with one parent interrupting cancel boundary attached to that same transaction node.",
        vec![
            "If this is not a real BPMN transaction cancel path, replace `<bpmn:cancelEventDefinition>` with a regular `<bpmn:endEvent>`.".to_string(),
            "If it is a real transaction cancel path, move the cancel end inside one `<bpmn:transaction>` body and add the matching parent cancel boundary.".to_string(),
        ],
        format!(
            "Repair process '{process_id}' so end event '{node_id}' no longer uses `<bpmn:cancelEventDefinition>` outside one bounded transaction shell. Either replace it with a regular end event, or move it inside one `<bpmn:transaction>` body and add the matching parent interrupting cancel boundary attached to that transaction."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn multiple_transaction_cancel_end_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::from_parts(
        "bpmn.unsupported_transaction_configuration",
        "Transaction shell supports only one cancel end event",
        format!(
            "Process '{process_id}' transaction node '{node_id}' contains more than one nested cancel end event."
        ),
        "The bounded transaction-cancel slice supports exactly one nested `<bpmn:endEvent>` carrying `<bpmn:cancelEventDefinition>` inside one transaction shell, so the engine can map that path to one parent interrupting cancel boundary deterministically.",
        vec![
            "Keep at most one nested cancel end inside this `<bpmn:transaction>` body.".to_string(),
            "If multiple cancel outcomes are needed, merge them through internal gateways and route them into one shared cancel end.".to_string(),
        ],
        format!(
            "Repair transaction node '{node_id}' in process '{process_id}' so its bounded `<bpmn:transaction>` body contains exactly one nested cancel end event with `<bpmn:cancelEventDefinition>`. Preserve workflow intent, but merge multiple cancel exits into one bounded cancel path."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn transaction_cancel_missing_boundary_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::from_parts(
        "bpmn.unsupported_transaction_configuration",
        "Transaction cancel path is missing the parent cancel boundary",
        format!(
            "Process '{process_id}' transaction node '{node_id}' contains a nested cancel end but does not expose a matching parent interrupting cancel boundary."
        ),
        "The bounded engine only executes transaction cancel semantics when one transaction shell has both sides of the path: one nested cancel end inside the child body and one parent interrupting cancel boundary attached to that same transaction node.",
        vec![
            "Add one interrupting `boundaryEvent` with `<bpmn:cancelEventDefinition>` attached to this `<bpmn:transaction>` node.".to_string(),
            "Keep the boundary's outgoing sequence flow as the parent cancel route, instead of letting the transaction fall through its normal success path.".to_string(),
        ],
        format!(
            "Repair transaction node '{node_id}' in process '{process_id}' so its bounded cancel path is complete: keep exactly one nested cancel end inside the `<bpmn:transaction>` body and add one parent interrupting `boundaryEvent` with `<bpmn:cancelEventDefinition>` attached to that same transaction node, routing the cancel path through the boundary's outgoing sequence flow."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn transaction_error_missing_boundary_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::from_parts(
        "bpmn.unsupported_transaction_configuration",
        "Transaction error path is missing the parent error boundary",
        format!(
            "Process '{process_id}' transaction node '{node_id}' contains a nested error end but does not expose any matching parent interrupting error boundary."
        ),
        "The bounded engine only executes transaction error semantics when one transaction shell has both sides of the path: one or more nested error ends inside the child body and one or more matching parent interrupting error boundaries attached to that same transaction node. If a boundary carries `errorRef`, it must match the thrown error; if it omits `errorRef`, it acts as the bounded catch-all path.",
        vec![
            "Add one or more interrupting `boundaryEvent` nodes with `<bpmn:errorEventDefinition>` attached to this `<bpmn:transaction>` node.".to_string(),
            "For every nested error end that uses `errorRef`, either copy that same `errorRef` to one or more matching boundaries or omit `errorRef` on one boundary to make it the bounded catch-all path.".to_string(),
        ],
        format!(
            "Repair transaction node '{node_id}' in process '{process_id}' so its bounded error path is complete: keep one or more nested error ends inside the `<bpmn:transaction>` body and add one or more parent interrupting `boundaryEvent` nodes with `<bpmn:errorEventDefinition>` attached to that same transaction node, routing each thrown error through every selected boundary's outgoing sequence flow. If a nested error end declares `errorRef`, make one or more boundaries use the same `errorRef` or omit `errorRef` on one boundary to catch that error generically."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn generic_transaction_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::from_parts(
        "bpmn.unsupported_transaction_configuration",
        "Transaction configuration exceeds the bounded slice",
        format!(
            "Process '{process_id}' transaction node '{node_id}' uses unsupported configuration '{detail}'."
        ),
        "The current engine supports only one bounded transaction shell shape: exactly one nested start event, at least one nested end event, at most one bounded cancel path, and one or more bounded error-end paths, with every thrown error paired to one or more matching parent interrupting error boundaries on that same transaction owner.",
        vec![
            "Keep the transaction intent, but reduce it to the bounded transaction shell shape.".to_string(),
            "If the model depends on richer BPMN transaction features such as broader throw-compensation forms, compensation event subprocesses, or default compensation, preserve that requirement explicitly and defer execution until support lands.".to_string(),
        ],
        format!(
            "Rewrite transaction node '{node_id}' in process '{process_id}' so it fits the bounded slice: one `<bpmn:transaction>` shell with exactly one nested `<bpmn:startEvent>`, at least one nested `<bpmn:endEvent>`, at most one bounded cancel path, and one or more bounded error-end paths, with every thrown error paired to one or more matching parent interrupting error boundaries on that transaction owner. Preserve workflow intent, but remove unsupported configuration '{detail}'."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}
