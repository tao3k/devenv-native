use super::boundary::boundary_configuration_issue;
use crate::error::BpmnEngineError;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn issue_from_bpmn_topology_error(error: &BpmnEngineError) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::UnknownBoundaryAttachment {
            process_id,
            node_id,
            attached_to_node_id,
        } => LintIssue::from_parts(
            "bpmn.unknown_boundary_attachment",
            "Boundary event attaches to an unknown node",
            format!(
                "Process '{process_id}' boundary event '{node_id}' references missing attached node '{attached_to_node_id}'."
            ),
            "The bounded engine can only normalize a boundary event when `attachedToRef` points to an existing supported owner in the same process.",
            vec![
                format!("Change `attachedToRef` on boundary event '{node_id}' to an existing node id in process '{process_id}'."),
                "Attach the boundary only to one supported owner: one host-blocking task for interrupting timer, message, signal, or conditional boundaries, one bounded embedded `<bpmn:subProcess>` owner or one bounded same-package `<bpmn:callActivity>` owner for one interrupting timer, message, signal, or conditional boundary with the bounded mixed-owner subset of one or more interrupting error boundaries, one bounded `<bpmn:transaction>` shell for one interrupting timer/message/signal/conditional boundary either on its own, with one interrupting cancel boundary, with one or more interrupting error boundaries, or with one interrupting cancel boundary plus one or more interrupting error boundaries, one non-repeating or bounded standard-loop, sequential multi-instance, or parallel multi-instance host-blocking task for non-interrupting timer, message, signal, or conditional boundaries, one bounded `<bpmn:transaction>` shell for cancel boundaries, or one bounded `<bpmn:transaction>` / embedded `<bpmn:subProcess>` / same-package `<bpmn:callActivity>` owner for error boundaries.".to_string(),
            ],
            format!(
                "Edit process '{process_id}' so boundary event '{node_id}' uses an `attachedToRef` that points to an existing supported owner in the same process. Preserve workflow intent, but do not leave the boundary attached to missing node '{attached_to_node_id}'."
            ),
            json!({
                "process_id": process_id,
                "node_id": node_id,
                "attached_to_node_id": attached_to_node_id,
            }),
        ),
        BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id,
            node_id,
            detail,
        } => return Some(boundary_configuration_issue(process_id, node_id, detail)),
        BpmnEngineError::UnsupportedEventBasedGatewayConfiguration {
            process_id,
            node_id,
            detail,
        } => LintIssue::from_parts(
            "bpmn.unsupported_event_based_gateway_configuration",
            "Event-based gateway configuration exceeds the bounded slice",
            format!(
                "Process '{process_id}' event-based gateway '{node_id}' uses unsupported configuration '{detail}'."
            ),
            "The current engine supports only one bounded event-based gateway shape: one exclusive eventBasedGateway whose outgoing paths all target message, signal, timer, or conditional intermediate catch events.",
            vec![
                "Keep the winner-takes-all intent, but make every outgoing branch from the eventBasedGateway point to one intermediateCatchEvent.".to_string(),
                "Use only messageEventDefinition, signalEventDefinition, timerEventDefinition, or conditionalEventDefinition with one bounded condition on those waiting nodes in this bounded slice.".to_string(),
            ],
            format!(
                "Rewrite event-based gateway '{node_id}' in process '{process_id}' so it fits the bounded slice: use one exclusive `eventBasedGateway` with at least two outgoing branches, and make every outgoing target one `intermediateCatchEvent` with exactly one `messageEventDefinition`, `signalEventDefinition`, `timerEventDefinition`, or `conditionalEventDefinition` with one bounded condition. Preserve workflow intent, but remove unsupported configuration '{detail}'."
            ),
            json!({
                "process_id": process_id,
                "node_id": node_id,
                "detail": detail,
            }),
        ),
        _ => return None,
    })
}
