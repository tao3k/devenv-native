use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn subprocess_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    match detail {
        "event_subprocess" => event_subprocess_issue(process_id, node_id, detail),
        "embedded_subprocess_start_event_count" => {
            embedded_subprocess_start_event_issue(process_id, node_id, detail)
        }
        "transaction_start_event_count" => {
            transaction_start_event_issue(process_id, node_id, detail)
        }
        "embedded_subprocess_missing_end_event" => {
            embedded_subprocess_missing_end_issue(process_id, node_id, detail)
        }
        "transaction_missing_end_event" => {
            transaction_missing_end_issue(process_id, node_id, detail)
        }
        "recursive_call_activity" => recursive_subprocess_issue(process_id, node_id, detail),
        _ => generic_subprocess_configuration_issue(process_id, node_id, detail),
    }
}

fn event_subprocess_issue(process_id: &str, node_id: &str, detail: &'static str) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_subprocess_configuration",
        "Event subprocesses are deferred",
        format!(
            "Process '{process_id}' subprocess node '{node_id}' uses `triggeredByEvent=\"true\"`, which is outside the bounded slice."
        ),
        "The current engine supports one bounded embedded `subProcess` body, one bounded `<transaction>` shell, and one bounded non-recursive `callActivity`. It does not support event subprocesses, including compensation event subprocesses.",
        vec![
            "If the nested flow is not meant to be event-triggered, remove `triggeredByEvent=\"true\"` and keep a bounded embedded `subProcess` with exactly one nested `startEvent` and at least one nested `endEvent`.".to_string(),
            "If the model depends on triggered interruption, remodel it with the currently supported boundary-event or transaction-boundary subset instead of an event subprocess.".to_string(),
            "Do not rely on compensation event subprocesses or other `triggeredByEvent=\"true\"` subprocess forms in this bounded slice.".to_string(),
        ],
        format!(
            "Rewrite subprocess node '{node_id}' in process '{process_id}' so it no longer uses `triggeredByEvent=\"true\"`. Preserve workflow intent, but either remodel it as a bounded embedded subprocess or move the triggered behavior into the supported boundary-event or transaction-boundary subset."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn embedded_subprocess_start_event_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_subprocess_configuration",
        "Embedded subprocess must have exactly one start event",
        format!(
            "Process '{process_id}' subprocess node '{node_id}' contains an embedded subprocess body without exactly one nested start event."
        ),
        "The bounded embedded `subProcess` slice follows the upstream `SpiffWorkflow` rule that an inline subprocess body must contain exactly one nested `startEvent` before the engine can materialize it as one child process.",
        vec![
            "Keep exactly one nested `<bpmn:startEvent>` inside the embedded `subProcess` body.".to_string(),
            "If the current model has multiple entry points, rewrite them into one bounded start path and move branching into downstream gateways or tasks.".to_string(),
        ],
        format!(
            "Repair subprocess node '{node_id}' in process '{process_id}' so its embedded `subProcess` body contains exactly one nested `<bpmn:startEvent>`. Preserve workflow intent, but merge or remove extra entry points instead of leaving zero or multiple start events."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn transaction_start_event_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_subprocess_configuration",
        "Transaction shell must have exactly one start event",
        format!(
            "Process '{process_id}' transaction node '{node_id}' contains a bounded transaction body without exactly one nested start event."
        ),
        "The bounded transaction shell follows the same upstream nested-process entry rule as embedded subprocesses: the engine must see exactly one nested `startEvent` before it can materialize the transaction body as one child process frame.",
        vec![
            "Keep exactly one nested `<bpmn:startEvent>` inside the `<bpmn:transaction>` body.".to_string(),
            "If the model currently has multiple transaction entry points, merge them into one start path and move branching into downstream gateways or tasks.".to_string(),
        ],
        format!(
            "Repair transaction node '{node_id}' in process '{process_id}' so its bounded `<bpmn:transaction>` body contains exactly one nested `<bpmn:startEvent>`. Preserve workflow intent, but merge or remove extra entry points instead of leaving zero or multiple start events."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn embedded_subprocess_missing_end_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_subprocess_configuration",
        "Embedded subprocess is missing an end event",
        format!(
            "Process '{process_id}' subprocess node '{node_id}' contains an embedded subprocess body without any nested end event."
        ),
        "The bounded embedded `subProcess` slice requires at least one nested `endEvent` so the child process can complete and return to the parent frame deterministically.",
        vec![
            "Add at least one nested `<bpmn:endEvent>` inside the embedded `subProcess` body.".to_string(),
            "Reconnect the last internal task or gateway so the embedded subprocess can reach that end event deterministically.".to_string(),
        ],
        format!(
            "Repair subprocess node '{node_id}' in process '{process_id}' so its embedded `subProcess` body contains at least one nested `<bpmn:endEvent>` and internal sequence flows can reach it."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn transaction_missing_end_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_subprocess_configuration",
        "Transaction shell is missing an end event",
        format!(
            "Process '{process_id}' transaction node '{node_id}' contains a bounded transaction body without any nested end event."
        ),
        "The bounded transaction shell still needs at least one nested `endEvent` so the child process frame can complete and return to the parent process deterministically.",
        vec![
            "Add at least one nested `<bpmn:endEvent>` inside the `<bpmn:transaction>` body.".to_string(),
            "Reconnect the last internal task or gateway so the transaction shell can reach that end event deterministically.".to_string(),
        ],
        format!(
            "Repair transaction node '{node_id}' in process '{process_id}' so its bounded `<bpmn:transaction>` body contains at least one nested `<bpmn:endEvent>` and internal sequence flows can reach it."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn recursive_subprocess_issue(process_id: &str, node_id: &str, detail: &'static str) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_subprocess_configuration",
        "Recursive subprocess call chain is unsupported",
        format!(
            "Process '{process_id}' subprocess node '{node_id}' participates in a recursive subprocess or call-activity chain."
        ),
        "The bounded engine now supports one embedded `subProcess` body, one bounded `<transaction>` shell, and one bounded same-package `callActivity`, but it still rejects recursive nested execution graphs because they break the current bounded frame model.",
        vec![
            "Keep the nested workflow intent, but remove the cycle so subprocess execution becomes acyclic.".to_string(),
            "If you need reuse, point `calledElement` at a different non-recursive process instead of bouncing back into an ancestor or the same process.".to_string(),
        ],
        format!(
            "Rewrite subprocess node '{node_id}' in process '{process_id}' so nested execution is acyclic. Preserve workflow intent, but do not let embedded subprocesses or `callActivity` targets recurse back into the same process chain."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn generic_subprocess_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_subprocess_configuration",
        "Subprocess configuration exceeds the bounded slice",
        format!(
            "Process '{process_id}' subprocess node '{node_id}' uses unsupported configuration '{detail}'."
        ),
        "The current engine supports one bounded embedded `subProcess` body, one bounded `<transaction>` shell, and one bounded non-recursive `callActivity` that targets another process in the same BPMN package. Both nested inline shells require exactly one nested start event and at least one nested end event.",
        vec![
            "Keep the nested workflow intent, but reduce the subprocess shape to the bounded embedded, bounded transaction-shell, or non-recursive call-activity subset.".to_string(),
            "If the subprocess uses deferred nested BPMN features, preserve intent while rewriting it into the supported bounded structure.".to_string(),
        ],
        format!(
            "Rewrite subprocess node '{node_id}' in process '{process_id}' so it fits the bounded slice: either one embedded `subProcess` body with exactly one nested `startEvent` and at least one nested `endEvent`, one bounded `<transaction>` shell with exactly one nested `startEvent` and at least one nested `endEvent`, or one non-recursive `callActivity` with a valid `calledElement` that points to another executable process in the same BPMN package. Preserve workflow intent, but remove unsupported configuration '{detail}'."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}
