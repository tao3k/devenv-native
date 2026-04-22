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
        "embedded_subprocess_error_missing_boundary" => {
            embedded_subprocess_error_missing_boundary_issue(process_id, node_id, detail)
        }
        "call_activity_error_missing_boundary" => {
            call_activity_error_missing_boundary_issue(process_id, node_id, detail)
        }
        "error_end_requires_supported_error_owner"
        | "error_end_requires_supported_subprocess_shell" => {
            error_end_requires_supported_error_owner_issue(process_id, node_id, detail)
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

fn embedded_subprocess_error_missing_boundary_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_subprocess_configuration",
        "Embedded subprocess error path is missing the parent error boundary",
        format!(
            "Process '{process_id}' subprocess node '{node_id}' contains one or more nested error ends but does not expose any matching parent interrupting error boundary."
        ),
        "The bounded engine supports an embedded-subprocess error path only when one bounded embedded `subProcess` owner carries one or more interrupting error boundaries that match each nested error end, including one optional catch-all boundary with omitted `errorRef`.",
        vec![
            "Add one or more interrupting `boundaryEvent` nodes with `<bpmn:errorEventDefinition>` attached to this embedded `<bpmn:subProcess>` node.".to_string(),
            "For every nested error end that declares `errorRef`, make one or more parent error boundaries use the same `errorRef` or omit `errorRef` on one boundary to catch that error generically.".to_string(),
        ],
        format!(
            "Repair subprocess node '{node_id}' in process '{process_id}' so its bounded embedded error path is complete: keep one or more nested error ends inside the embedded `<bpmn:subProcess>` body and add one or more parent interrupting `boundaryEvent` nodes with `<bpmn:errorEventDefinition>` attached to that same subprocess node, routing each thrown error through every selected boundary's outgoing sequence flow. If a nested error end declares `errorRef`, make one or more boundaries use the same `errorRef` or omit `errorRef` on one boundary to catch that error generically."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn call_activity_error_missing_boundary_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_subprocess_configuration",
        "Call activity error path is missing the parent error boundary",
        format!(
            "Process '{process_id}' call activity '{node_id}' targets a child process that contains one or more error ends but does not expose any matching parent interrupting error boundary."
        ),
        "The bounded engine only executes same-package `callActivity` error semantics when both sides of the path exist: the called process may finish through one or more error ends, and the parent `callActivity` owner must expose one or more matching interrupting error boundaries. If a boundary carries `errorRef`, it must match the thrown error; if it omits `errorRef`, it acts as the bounded catch-all path.",
        vec![
            "Add one or more interrupting `boundaryEvent` nodes with `<bpmn:errorEventDefinition>` attached to this bounded same-package `<bpmn:callActivity>` node.".to_string(),
            "For every child-process error end that uses `errorRef`, either copy that same `errorRef` to one or more matching boundaries or omit `errorRef` on one boundary to make it the bounded catch-all path.".to_string(),
        ],
        format!(
            "Repair call activity '{node_id}' in process '{process_id}' so its bounded same-package error path is complete: keep one or more error ends in the called process and add one or more parent interrupting `boundaryEvent` nodes with `<bpmn:errorEventDefinition>` attached to that same `<bpmn:callActivity>` owner, routing each thrown error through every selected boundary's outgoing sequence flow. If a child-process error end declares `errorRef`, make one or more boundaries use the same `errorRef` or omit `errorRef` on one boundary to catch that error generically."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn error_end_requires_supported_error_owner_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_subprocess_configuration",
        "Error end event must belong to a supported error path",
        format!(
            "Process '{process_id}' end event '{node_id}' uses `<errorEventDefinition>` outside one bounded transaction shell, one bounded embedded subprocess shell, or one child process reached by a bounded same-package call activity with matching parent error boundaries."
        ),
        "The bounded engine now supports two bounded error-end families: one top-level executable process may terminate the instance directly in failed state, and one subprocess-owned error path may route through a bounded transaction shell, bounded embedded `<bpmn:subProcess>` shell, or one called process entered only through a bounded same-package `callActivity` owner carrying matching interrupting error boundaries.",
        vec![
            "If this should terminate the whole workflow, keep the `<bpmn:errorEventDefinition>` end event in one executable top-level process and let the instance fail terminally.".to_string(),
            "If this is not a real top-level or subprocess-owned error path, replace `<bpmn:errorEventDefinition>` with a regular `<bpmn:endEvent>`.".to_string(),
            "If it is a real bounded subprocess error path, move the error end under one supported owner and add the matching parent interrupting error boundaries on that same owner.".to_string(),
        ],
        format!(
            "Repair process '{process_id}' so end event '{node_id}' uses `<bpmn:errorEventDefinition>` only in one bounded supported error path. Either keep it in one executable top-level process so the instance fails terminally, replace it with a regular end event, move it inside one bounded `<bpmn:transaction>` shell, move it inside one bounded embedded `<bpmn:subProcess>` shell, or keep it in a called process that is entered only through one bounded same-package `<bpmn:callActivity>` owner carrying matching parent interrupting error boundaries."
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
