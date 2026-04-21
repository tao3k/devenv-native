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
        "The current engine only executes bounded compensation as part of transaction-cancel handling. Compensation boundaries and `isForCompensation=\"true\"` handler activities must therefore live inside one nested `<bpmn:transaction>` body.",
        vec![
            "Move the compensation boundary and handler into one bounded `<bpmn:transaction>` shell if transaction cancel semantics are really required.".to_string(),
            "If transaction cancel semantics are not required, remove the compensation markers and rewrite the flow with ordinary tasks and sequenceFlow routing.".to_string(),
        ],
        format!(
            "Rewrite process '{process_id}' so compensation node '{node_id}' no longer uses compensation semantics outside one bounded `<bpmn:transaction>` shell. Preserve workflow intent, but keep bounded compensation only inside one transaction body whose cancel path can trigger the handler."
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
        "The current engine executes compensation only through one explicit boundary-to-handler binding inside one transaction shell during transaction cancel. It does not execute `activityRef`-targeted throw compensation directly from end events.",
        vec![
            "If transaction rollback is really required, remodel the flow with one bounded transaction cancel path plus explicit compensation boundary-to-handler bindings on the completed activities.".to_string(),
            "If the flow only needs terminal completion, remove `<compensateEventDefinition>` and keep an ordinary `<bpmn:endEvent>`.".to_string(),
            "Do not rely on `activityRef`-targeted compensation on throwing end events in this bounded slice.".to_string(),
        ],
        format!(
            "Rewrite compensation node '{node_id}' in process '{process_id}' so it no longer throws compensation from `<bpmn:endEvent>`. Preserve workflow intent, but either use an ordinary end event or remodel the compensation behavior as explicit boundary-to-handler bindings inside one bounded transaction cancel path."
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
        "The current engine does not execute default compensation from throwing end events. It only executes explicit boundary-to-handler compensation bindings inside one transaction shell during transaction cancel.",
        vec![
            "If transaction rollback is really required, remodel the flow with one bounded transaction cancel path plus explicit compensation boundary-to-handler bindings on the completed activities.".to_string(),
            "If the flow only needs terminal completion, remove `<compensateEventDefinition>` and keep an ordinary `<bpmn:endEvent>`.".to_string(),
            "Do not rely on omitted `activityRef` default compensation on throwing end events in this bounded slice.".to_string(),
        ],
        format!(
            "Rewrite compensation node '{node_id}' in process '{process_id}' so it no longer uses default compensation from `<bpmn:endEvent>`. Preserve workflow intent, but either use an ordinary end event or remodel the compensation behavior as explicit boundary-to-handler bindings inside one bounded transaction cancel path."
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
        "The current engine executes compensation only through one explicit boundary-to-handler binding inside one transaction shell during transaction cancel. It does not execute `activityRef`-targeted throw compensation directly from intermediate throw events.",
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
        "The current engine does not execute default compensation from throwing intermediate events. It only executes explicit boundary-to-handler compensation bindings inside one transaction shell during transaction cancel.",
        vec![
            "If transaction rollback is really required, remodel the flow with one bounded transaction cancel path plus explicit compensation boundary-to-handler bindings on the completed activities.".to_string(),
            "If the flow only needs ordinary progression, remove `<compensateEventDefinition>` and keep normal sequence-flow routing to the next supported node.".to_string(),
            "Do not rely on omitted `activityRef` default compensation on throwing intermediate events in this bounded slice.".to_string(),
        ],
        format!(
            "Rewrite compensation node '{node_id}' in process '{process_id}' so it no longer uses default compensation from `<bpmn:intermediateThrowEvent>`. Preserve workflow intent, but either restore ordinary sequence-flow progression or remodel the compensation behavior as explicit boundary-to-handler bindings inside one bounded transaction cancel path."
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
        "The current engine supports only one bounded compensation shape: inside one transaction shell, attach one compensation boundary event with `<bpmn:compensateEventDefinition>` to one completed serviceTask, userTask, manualTask, or businessRuleTask, connect that boundary to exactly one detached `isForCompensation=\"true\"` handler activity through one association, and let transaction cancel execute those handlers in reverse completion order without normal sequence-flow routing.",
        vec![
            "Keep compensation inside one transaction shell and attach each compensation boundary to exactly one direct host-blocking activity.".to_string(),
            "Make the handler a detached serviceTask, userTask, manualTask, or businessRuleTask marked with `isForCompensation=\"true\"`, and connect it with exactly one association from the compensation boundary.".to_string(),
            "Do not place normal sequence flows, loops, multi-instance characteristics, throw compensation events, or compensation event subprocesses on this bounded compensation path.".to_string(),
        ],
        format!(
            "Repair compensation node '{node_id}' in process '{process_id}' so it fits the bounded compensation slice: keep it inside one `<bpmn:transaction>` shell, use exactly one compensation boundary with `<bpmn:compensateEventDefinition>` attached to one direct host-blocking activity, connect that boundary to exactly one detached handler activity marked `isForCompensation=\"true\"` through one association, and remove unsupported configuration '{detail}'."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}
