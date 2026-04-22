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
        "async_throw_compensation_end_event" => {
            async_throw_compensation_end_event_issue(process_id, node_id, detail)
        }
        "throw_compensation_intermediate_event" => {
            throw_compensation_intermediate_event_issue(process_id, node_id, detail)
        }
        "async_throw_compensation_intermediate_event" => {
            async_throw_compensation_intermediate_event_issue(process_id, node_id, detail)
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
        "The current engine only executes bounded compensation inside one nested `<bpmn:transaction>` body: either through explicit boundary-to-handler replay during transaction cancel, one synchronous throw-compensation end event that either targets one explicit `activityRef` or defaults to replaying every already compensable activity, or one synchronous throw-compensation intermediate event that either references one already compensable activity through explicit `activityRef` or omits `activityRef` to replay every already compensable activity in that same transaction shell before normal sequence-flow routing resumes.",
        vec![
            "Move the compensation boundary and handler into one bounded `<bpmn:transaction>` shell if compensation semantics are really required.".to_string(),
            "If transaction cancel semantics are not required, remove the compensation markers and rewrite the flow with ordinary tasks and sequenceFlow routing.".to_string(),
        ],
        format!(
            "Rewrite process '{process_id}' so compensation node '{node_id}' no longer uses compensation semantics outside one bounded `<bpmn:transaction>` shell. Preserve workflow intent, but keep bounded compensation only inside one transaction body that owns the explicit compensation boundary-to-handler bindings and any supported synchronous throw-compensation end or intermediate events."
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
        "The current engine executes one bounded throw-compensation end-event subset: the event must run inside one supported transaction shell, must stay synchronous, and must either reference one explicit `activityRef` that reuses an already valid boundary-to-handler compensation binding for that target activity or omit `activityRef` to replay every already compensable activity in reverse completion order. Any other end-event throw-compensation shape remains unsupported.",
        vec![
            "If compensation is really required, remodel the flow so the throwing end event lives inside one bounded transaction shell and stays synchronous.".to_string(),
            "Use one explicit `activityRef` when only one already compensable activity should replay, or omit `activityRef` when every already compensable activity in the transaction shell should replay in reverse completion order.".to_string(),
            "If the flow only needs terminal completion, remove `<compensateEventDefinition>` and keep an ordinary `<bpmn:endEvent>`.".to_string(),
            "Do not rely on throw-compensation end events outside the bounded transaction-shell subset in this slice.".to_string(),
        ],
        format!(
            "Rewrite compensation node '{node_id}' in process '{process_id}' so its `<bpmn:endEvent>` throw-compensation shape fits the bounded subset: place it inside one supported `<bpmn:transaction>` shell, keep it synchronous, and either use one explicit `activityRef` that targets one already compensable activity or omit `activityRef` so the bounded transaction shell replays every already compensable activity in reverse completion order. Otherwise, preserve workflow intent with an ordinary end event."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn async_throw_compensation_end_event_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_compensation_configuration",
        "Asynchronous throw compensation end events are deferred",
        format!(
            "Process '{process_id}' compensation node '{node_id}' uses `<endEvent><compensateEventDefinition waitForCompletion=\"false\" ... /></endEvent>`, which is outside the bounded slice."
        ),
        "The current engine supports only synchronous throw-compensation end-event behavior inside one bounded transaction shell, whether the event targets one explicit `activityRef` or defaults to replaying every already compensable activity. It does not support fire-and-continue compensation from an end event.",
        vec![
            "Keep the throw-compensation end event synchronous by omitting `waitForCompletion` or setting it to `true`.".to_string(),
            "If the flow only needs terminal completion, remove `<compensateEventDefinition>` and keep an ordinary `<bpmn:endEvent>`.".to_string(),
        ],
        format!(
            "Rewrite compensation node '{node_id}' in process '{process_id}' so its throw-compensation end event stays synchronous. Preserve workflow intent, but omit `waitForCompletion` or set it to `true`, and keep the event inside the bounded transaction-shell subset."
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
        "The current engine executes one bounded throw-compensation intermediate-event subset: the event must run inside one supported transaction shell, must stay synchronous, and must either reference one explicit `activityRef` that reuses an already valid boundary-to-handler compensation binding for that target activity or omit `activityRef` to replay every already compensable activity in reverse completion order before normal sequence-flow routing resumes. Any other intermediate throw-compensation shape remains unsupported.",
        vec![
            "If compensation is really required, place the throwing intermediate event inside one bounded transaction shell and either target one activity that already owns an explicit compensation boundary-to-handler binding or omit `activityRef` to replay every already compensable activity in reverse completion order.".to_string(),
            "Keep the event synchronous and preserve ordinary sequence-flow routing after compensation completes.".to_string(),
            "Do not rely on throw-compensation intermediate events outside the bounded transaction-shell subset in this slice.".to_string(),
        ],
        format!(
            "Rewrite compensation node '{node_id}' in process '{process_id}' so its `<bpmn:intermediateThrowEvent>` throw-compensation shape fits the bounded subset: place it inside one supported `<bpmn:transaction>` shell, keep it synchronous, and either use one explicit `activityRef` that targets one activity with an explicit boundary-to-handler compensation binding or omit `activityRef` so the bounded transaction shell replays every already compensable activity in reverse completion order before ordinary sequence-flow routing resumes. Otherwise, preserve workflow intent with ordinary sequence-flow routing."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn async_throw_compensation_intermediate_event_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_compensation_configuration",
        "Asynchronous throw compensation intermediate events are deferred",
        format!(
            "Process '{process_id}' compensation node '{node_id}' uses `<intermediateThrowEvent><compensateEventDefinition waitForCompletion=\"false\" ... /></intermediateThrowEvent>`, which is outside the bounded slice."
        ),
        "The current engine supports only synchronous throw-compensation behavior for intermediate throw events inside one bounded transaction shell, whether the event uses one explicit `activityRef` or omits `activityRef` for bounded default replay. It does not support fire-and-continue compensation from an intermediate event.",
        vec![
            "Keep the throw-compensation intermediate event synchronous by omitting `waitForCompletion` or setting it to `true`.".to_string(),
            "If the flow only needs ordinary progression, remove `<compensateEventDefinition>` and keep normal sequence-flow routing to the next supported node.".to_string(),
        ],
        format!(
            "Rewrite compensation node '{node_id}' in process '{process_id}' so its throw-compensation intermediate event stays synchronous. Preserve workflow intent, but omit `waitForCompletion` or set it to `true`, and keep the event inside the bounded transaction-shell subset."
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
        "The current engine supports only three bounded compensation shapes inside one transaction shell: one explicit boundary-to-handler binding whose handlers can replay in reverse completion order during transaction cancel, one synchronous throw-compensation end event that either targets one already compensable activity through explicit `activityRef` or replays every already compensable activity by default, and one synchronous throw-compensation intermediate event that either targets one already compensable activity through explicit `activityRef` or replays every already compensable activity by default before normal sequence-flow routing resumes.",
        vec![
            "Keep compensation inside one transaction shell and attach each compensation boundary to exactly one direct host-blocking activity.".to_string(),
            "Make the handler a detached serviceTask, userTask, manualTask, or businessRuleTask marked with `isForCompensation=\"true\"`, and connect it with exactly one association from the compensation boundary.".to_string(),
            "Use throw-compensation end or intermediate events only when they stay synchronous. End or intermediate events may target one explicit `activityRef` or omit `activityRef` for bounded default replay; any explicit target must already own a valid compensation boundary-to-handler binding.".to_string(),
            "Do not place loops, multi-instance characteristics, compensation event subprocesses, or asynchronous throw-compensation forms on this bounded compensation path.".to_string(),
        ],
        format!(
            "Repair compensation node '{node_id}' in process '{process_id}' so it fits the bounded compensation slice: keep it inside one `<bpmn:transaction>` shell, use explicit boundary-to-handler bindings for compensable activities, and only use synchronous throw-compensation end or intermediate events. End or intermediate events may target one explicit `activityRef` or omit `activityRef` for bounded default replay; any explicit target must already own a valid compensation handler binding. Remove unsupported configuration '{detail}'."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}
