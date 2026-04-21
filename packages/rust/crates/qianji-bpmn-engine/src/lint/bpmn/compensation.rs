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
        "default_compensation_end_event" => {
            default_compensation_end_event_issue(process_id, node_id, detail)
        }
        "throw_compensation_intermediate_event" => {
            throw_compensation_intermediate_event_issue(process_id, node_id, detail)
        }
        "default_compensation_intermediate_event" => {
            default_compensation_intermediate_event_issue(process_id, node_id, detail)
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
        "The current engine only executes bounded compensation inside one nested `<bpmn:transaction>` body: either through explicit boundary-to-handler replay during transaction cancel or through one synchronous targeted throw-compensation end event that references an already compensable activity in that same transaction shell.",
        vec![
            "Move the compensation boundary and handler into one bounded `<bpmn:transaction>` shell if compensation semantics are really required.".to_string(),
            "If transaction cancel semantics are not required, remove the compensation markers and rewrite the flow with ordinary tasks and sequenceFlow routing.".to_string(),
        ],
        format!(
            "Rewrite process '{process_id}' so compensation node '{node_id}' no longer uses compensation semantics outside one bounded `<bpmn:transaction>` shell. Preserve workflow intent, but keep bounded compensation only inside one transaction body that owns the explicit compensation boundary-to-handler bindings and any supported synchronous throw-compensation end events."
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
        "Throw compensation end events are deferred",
        format!(
            "Process '{process_id}' compensation node '{node_id}' uses `<endEvent><compensateEventDefinition ... /></endEvent>`, which is outside the bounded slice."
        ),
        "The current engine executes only one bounded throw-compensation end-event subset: the event must run inside one supported transaction shell, must reference one explicit `activityRef`, and must reuse an already valid boundary-to-handler compensation binding for that target activity. Any other end-event throw-compensation shape remains unsupported.",
        vec![
            "If compensation is really required, remodel the flow so the throwing end event lives inside one bounded transaction shell and targets one activity that already owns an explicit compensation boundary-to-handler binding.".to_string(),
            "If the flow only needs terminal completion, remove `<compensateEventDefinition>` and keep an ordinary `<bpmn:endEvent>`.".to_string(),
            "Do not rely on throw-compensation end events outside the bounded transaction-shell subset in this slice.".to_string(),
        ],
        format!(
            "Rewrite compensation node '{node_id}' in process '{process_id}' so its `<bpmn:endEvent>` throw-compensation shape fits the bounded subset: place it inside one supported `<bpmn:transaction>` shell, keep one explicit `activityRef`, and target one activity that already has an explicit boundary-to-handler compensation binding. Otherwise, preserve workflow intent with an ordinary end event."
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
        "The current engine supports only the default synchronous throw-compensation behavior. It does not support fire-and-continue compensation from an end event.",
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

fn default_compensation_end_event_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_compensation_configuration",
        "Default compensation end events are deferred",
        format!(
            "Process '{process_id}' compensation node '{node_id}' uses `<endEvent><compensateEventDefinition /></endEvent>` without `activityRef`, which implies default compensation and is outside the bounded slice."
        ),
        "The current engine does not execute default compensation from throwing end events. It only executes two bounded compensation shapes inside one transaction shell: explicit boundary-to-handler replay during transaction cancel and one synchronous targeted throw-compensation end event with explicit `activityRef`.",
        vec![
            "If transaction rollback is really required, remodel the flow with one bounded transaction cancel path plus explicit compensation boundary-to-handler bindings on the completed activities.".to_string(),
            "If the flow only needs terminal completion, remove `<compensateEventDefinition>` and keep an ordinary `<bpmn:endEvent>`.".to_string(),
            "Do not rely on omitted `activityRef` default compensation on throwing end events in this bounded slice.".to_string(),
        ],
        format!(
            "Rewrite compensation node '{node_id}' in process '{process_id}' so it no longer uses default compensation from `<bpmn:endEvent>`. Preserve workflow intent, but either use an ordinary end event or remodel the compensation behavior as one bounded transaction-shell compensation path using explicit boundary-to-handler bindings and, when needed, one synchronous targeted throw-compensation end event with explicit `activityRef`."
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
        "Throw compensation intermediate events are deferred",
        format!(
            "Process '{process_id}' compensation node '{node_id}' uses `<intermediateThrowEvent><compensateEventDefinition ... /></intermediateThrowEvent>`, which is outside the bounded slice."
        ),
        "The current engine executes compensation only through two bounded shapes inside one transaction shell: explicit boundary-to-handler replay during transaction cancel and one synchronous targeted throw-compensation end event. It does not execute `activityRef`-targeted throw compensation directly from intermediate throw events.",
        vec![
            "If transaction rollback is really required, remodel the flow with one bounded transaction cancel path plus explicit compensation boundary-to-handler bindings on the completed activities.".to_string(),
            "If the flow only needs ordinary progression, remove `<compensateEventDefinition>` and keep normal sequence-flow routing to the next supported node.".to_string(),
            "Do not rely on `activityRef`-targeted compensation on throwing intermediate events in this bounded slice.".to_string(),
        ],
        format!(
            "Rewrite compensation node '{node_id}' in process '{process_id}' so it no longer throws compensation from `<bpmn:intermediateThrowEvent>`. Preserve workflow intent, but either restore ordinary sequence-flow progression or remodel the compensation behavior as explicit boundary-to-handler bindings inside one bounded transaction cancel path."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn default_compensation_intermediate_event_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_compensation_configuration",
        "Default compensation intermediate events are deferred",
        format!(
            "Process '{process_id}' compensation node '{node_id}' uses `<intermediateThrowEvent><compensateEventDefinition /></intermediateThrowEvent>` without `activityRef`, which implies default compensation and is outside the bounded slice."
        ),
        "The current engine does not execute default compensation from throwing intermediate events. It only executes two bounded compensation shapes inside one transaction shell: explicit boundary-to-handler replay during transaction cancel and one synchronous targeted throw-compensation end event with explicit `activityRef`.",
        vec![
            "If transaction rollback is really required, remodel the flow with one bounded transaction cancel path plus explicit compensation boundary-to-handler bindings on the completed activities.".to_string(),
            "If the flow only needs ordinary progression, remove `<compensateEventDefinition>` and keep normal sequence-flow routing to the next supported node.".to_string(),
            "Do not rely on omitted `activityRef` default compensation on throwing intermediate events in this bounded slice.".to_string(),
        ],
        format!(
            "Rewrite compensation node '{node_id}' in process '{process_id}' so it no longer uses default compensation from `<bpmn:intermediateThrowEvent>`. Preserve workflow intent, but either restore ordinary sequence-flow progression or remodel the compensation behavior as one bounded transaction-shell compensation path using explicit boundary-to-handler bindings and, when needed, one synchronous targeted throw-compensation end event with explicit `activityRef`."
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
        "The current engine supports only two bounded compensation shapes inside one transaction shell: one explicit boundary-to-handler binding whose handlers can replay in reverse completion order during transaction cancel, and one synchronous throw-compensation end event that targets an already compensable activity through explicit `activityRef`.",
        vec![
            "Keep compensation inside one transaction shell and attach each compensation boundary to exactly one direct host-blocking activity.".to_string(),
            "Make the handler a detached serviceTask, userTask, manualTask, or businessRuleTask marked with `isForCompensation=\"true\"`, and connect it with exactly one association from the compensation boundary.".to_string(),
            "Use throw-compensation end events only when they stay synchronous, target one explicit `activityRef`, and point to one activity that already owns a valid compensation boundary-to-handler binding.".to_string(),
            "Do not place normal sequence flows, loops, multi-instance characteristics, intermediate throw compensation events, or compensation event subprocesses on this bounded compensation path.".to_string(),
        ],
        format!(
            "Repair compensation node '{node_id}' in process '{process_id}' so it fits the bounded compensation slice: keep it inside one `<bpmn:transaction>` shell, use explicit boundary-to-handler bindings for compensable activities, and only use throw-compensation end events when they stay synchronous and target one explicit `activityRef` that already owns a valid compensation handler binding. Remove unsupported configuration '{detail}'."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}
