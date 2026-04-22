use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn compensation_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    match detail {
        "compensation_requires_transaction_shell" => {
            compensation_requires_transaction_shell_issue(process_id, node_id, detail)
        }
        "throw_compensation_end_event" => {
            throw_compensation_end_event_issue(process_id, node_id, detail)
        }
        "throw_compensation_intermediate_event" => {
            throw_compensation_intermediate_event_issue(process_id, node_id, detail)
        }
        _ => generic_compensation_configuration_issue(process_id, node_id, detail),
    }
}

fn compensation_requires_transaction_shell_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_compensation_configuration",
        "Compensation is supported only inside a transaction shell",
        format!(
            "Process '{process_id}' compensation node '{node_id}' uses compensation semantics outside one bounded transaction shell."
        ),
        "The current engine only executes bounded compensation inside one nested `<bpmn:transaction>` body: either through explicit boundary-to-handler replay during transaction cancel, one throw-compensation end event that either stays synchronous or sets `waitForCompletion=\"false\"` while targeting one explicit `activityRef` or defaulting to every already compensable activity, or one throw-compensation intermediate event with the same bounded target/default replay subset and optional `waitForCompletion=\"false\"` fire-and-continue routing.",
        vec![
            "Move the compensation boundary and handler into one bounded `<bpmn:transaction>` shell if compensation semantics are really required.".to_string(),
            "If transaction cancel semantics are not required, remove the compensation markers and rewrite the flow with ordinary tasks and sequenceFlow routing.".to_string(),
        ],
        format!(
            "Rewrite process '{process_id}' so compensation node '{node_id}' no longer uses compensation semantics outside one bounded `<bpmn:transaction>` shell. Preserve workflow intent, but keep bounded compensation only inside one transaction body that owns the explicit compensation boundary-to-handler bindings and any supported throw-compensation end or intermediate events."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn throw_compensation_end_event_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_compensation_configuration",
        "Throw compensation end events outside the bounded subset are rejected",
        format!(
            "Process '{process_id}' compensation node '{node_id}' uses `<endEvent><compensateEventDefinition ... /></endEvent>`, which is outside the bounded slice."
        ),
        "The current engine executes one bounded throw-compensation end-event subset: the event must run inside one supported transaction shell and must either reference one explicit `activityRef` that reuses an already valid boundary-to-handler compensation binding for that target activity or omit `activityRef` to replay every already compensable activity in reverse completion order. The event may stay synchronous or set `waitForCompletion=\"false\"` so the parent scope resumes while the bounded compensation queue drains. Any other end-event throw-compensation shape remains unsupported.",
        vec![
            "If compensation is really required, remodel the flow so the throwing end event lives inside one bounded transaction shell.".to_string(),
            "Use one explicit `activityRef` when only one already compensable activity should replay, or omit `activityRef` when every already compensable activity in the transaction shell should replay in reverse completion order.".to_string(),
            "Choose synchronous behavior when parent completion should wait for replay, or set `waitForCompletion=\"false\"` when the parent scope should resume while the bounded compensation queue drains.".to_string(),
            "If the flow only needs terminal completion, remove `<compensateEventDefinition>` and keep an ordinary `<bpmn:endEvent>`.".to_string(),
            "Do not rely on throw-compensation end events outside the bounded transaction-shell subset in this slice.".to_string(),
        ],
        format!(
            "Rewrite compensation node '{node_id}' in process '{process_id}' so its `<bpmn:endEvent>` throw-compensation shape fits the bounded subset: place it inside one supported `<bpmn:transaction>` shell, and either use one explicit `activityRef` that targets one already compensable activity or omit `activityRef` so the bounded transaction shell replays every already compensable activity in reverse completion order. Choose synchronous behavior when parent completion should wait for replay, or set `waitForCompletion=\"false\"` when the parent scope should resume while the bounded compensation queue drains. Otherwise, preserve workflow intent with an ordinary end event."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn throw_compensation_intermediate_event_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_compensation_configuration",
        "Throw compensation intermediate events outside the bounded subset are rejected",
        format!(
            "Process '{process_id}' compensation node '{node_id}' uses `<intermediateThrowEvent><compensateEventDefinition ... /></intermediateThrowEvent>`, which is outside the bounded slice."
        ),
        "The current engine executes one bounded throw-compensation intermediate-event subset: the event must run inside one supported transaction shell and must either reference one explicit `activityRef` that reuses an already valid boundary-to-handler compensation binding for that target activity or omit `activityRef` to replay every already compensable activity in reverse completion order. The event may stay synchronous or set `waitForCompletion=\"false\"` so downstream routing continues while the compensation queue drains. Any other intermediate throw-compensation shape remains unsupported.",
        vec![
            "If compensation is really required, place the throwing intermediate event inside one bounded transaction shell and either target one activity that already owns an explicit compensation boundary-to-handler binding or omit `activityRef` to replay every already compensable activity in reverse completion order.".to_string(),
            "Choose synchronous behavior when downstream routing should wait for replay, or set `waitForCompletion=\"false\"` when downstream routing should continue while the bounded compensation queue drains.".to_string(),
            "Do not rely on throw-compensation intermediate events outside the bounded transaction-shell subset in this slice.".to_string(),
        ],
        format!(
            "Rewrite compensation node '{node_id}' in process '{process_id}' so its `<bpmn:intermediateThrowEvent>` throw-compensation shape fits the bounded subset: place it inside one supported `<bpmn:transaction>` shell, and either use one explicit `activityRef` that targets one activity with an explicit boundary-to-handler compensation binding or omit `activityRef` so the bounded transaction shell replays every already compensable activity in reverse completion order. Choose synchronous behavior when downstream routing should wait for replay, or set `waitForCompletion=\"false\"` when downstream routing should continue while the bounded compensation queue drains. Otherwise, preserve workflow intent with ordinary sequence-flow routing."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn generic_compensation_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_compensation_configuration",
        "Compensation configuration exceeds the bounded slice",
        format!(
            "Process '{process_id}' compensation node '{node_id}' uses unsupported configuration '{detail}'."
        ),
        "The current engine supports only three bounded compensation shapes inside one transaction shell: one explicit boundary-to-handler binding whose handlers can replay in reverse completion order during transaction cancel, one throw-compensation end event that either targets one already compensable activity through explicit `activityRef` or replays every already compensable activity by default while staying synchronous or using `waitForCompletion=\"false\"`, and one throw-compensation intermediate event that uses the same bounded target/default replay subset while staying synchronous or using `waitForCompletion=\"false\"` for fire-and-continue routing.",
        vec![
            "Keep compensation inside one transaction shell and attach each compensation boundary to exactly one direct host-blocking activity.".to_string(),
            "Make the handler a detached serviceTask, scriptTask, userTask, manualTask, or businessRuleTask marked with `isForCompensation=\"true\"`, and connect it with exactly one association from the compensation boundary.".to_string(),
            "Use throw-compensation end or intermediate events only inside one bounded transaction shell. Either throw shape may stay synchronous or set `waitForCompletion=\"false\"`, may target one explicit `activityRef`, or may omit `activityRef` for bounded default replay, and any explicit target must already own a valid compensation boundary-to-handler binding.".to_string(),
            "Do not place loops, multi-instance characteristics, or compensation event subprocesses on this bounded compensation path.".to_string(),
        ],
        format!(
            "Repair compensation node '{node_id}' in process '{process_id}' so it fits the bounded compensation slice: keep it inside one `<bpmn:transaction>` shell, use explicit boundary-to-handler bindings for compensable activities, and only use supported throw-compensation end or intermediate events. End or intermediate events may stay synchronous or set `waitForCompletion=\"false\"`, may target one explicit `activityRef`, or may omit `activityRef` for bounded default replay, and any explicit target must already own a valid compensation handler binding. Remove unsupported configuration '{detail}'."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}
