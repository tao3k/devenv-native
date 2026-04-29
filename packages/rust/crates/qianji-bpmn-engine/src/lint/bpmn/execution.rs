use super::compensation::compensation_configuration_issue;
use super::gateway::gateway_configuration_issue;
use super::subprocess::subprocess_configuration_issue;
use super::task::task_configuration_issue;
use super::transaction::transaction_configuration_issue;
use crate::error::BpmnEngineError;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn issue_from_bpmn_execution_shape_error(error: &BpmnEngineError) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::UnsupportedLoopConfiguration {
            process_id,
            node_id,
            detail,
        } => LintIssue::new(
            "bpmn.unsupported_loop_configuration",
            "Loop configuration exceeds the bounded slice",
            format!(
                "Process '{process_id}' loop node '{node_id}' uses unsupported configuration '{detail}'."
            ),
            "The current engine supports three bounded repeatable-task shapes: `standardLoopCharacteristics` on one serviceTask, scriptTask, userTask, manualTask, or businessRuleTask with a positive `loopMaximum` or one simple boolean loop condition, sequential `multiInstanceLoopCharacteristics isSequential=\"true\"` on those same host-blocking task kinds with either integer `loopCardinality` or collection-backed `loopDataInputRef` plus `inputDataItem`, and bounded parallel `multiInstanceLoopCharacteristics` with omitted or `isSequential=\"false\"` using that same bounded cardinality-or-collection expansion. Those multi-instance shapes may also carry one bounded optional output aggregation pair `loopDataOutputRef` plus `outputDataItem`, as long as the output path is different from the input path, and one bounded `completionCondition` using either one simple boolean variable path such as `approved` or `not approved`, or one counter comparison using `completed`, `active`, or `total` and their BPMN aliases `nrOfCompletedInstances`, `nrOfActiveInstances`, and `nrOfInstances`.",
            vec![
                "If you need bounded repeat execution now, rewrite the node to one serviceTask, scriptTask, userTask, manualTask, or businessRuleTask with either `standardLoopCharacteristics`, sequential `multiInstanceLoopCharacteristics isSequential=\"true\"`, or bounded parallel `multiInstanceLoopCharacteristics` with omitted or `isSequential=\"false\"`.".to_string(),
                "Use a positive `loopMaximum`, or one simple boolean condition like `done` or `not done`, or for multi-instance use either integer `loopCardinality` or one collection binding with `loopDataInputRef` plus `inputDataItem`. If you aggregate per-iteration output, provide both `loopDataOutputRef` and `outputDataItem`, keep the output path different from the input path, and for early completion keep `completionCondition` inside the bounded subset with one boolean variable path or one counter comparison like `completed >= 2`.".to_string(),
            ],
            format!(
                "Rewrite loop node '{node_id}' in process '{process_id}' so it fits the bounded slice: either use one `standardLoopCharacteristics` block on a serviceTask, scriptTask, userTask, manualTask, or businessRuleTask with a positive `loopMaximum` or one simple boolean loop condition like `done` or `not done`, or use one `multiInstanceLoopCharacteristics` block on those same host-blocking task kinds, setting `isSequential=\"true\"` for sequential execution or leaving it omitted or `isSequential=\"false\"` for bounded parallel execution, and choosing exactly one expansion mode: integer `loopCardinality`, or collection-backed `loopDataInputRef` plus `inputDataItem`. If you aggregate iteration output, keep `loopDataOutputRef` and `outputDataItem` paired and make the output path different from the input path. If you need multi-instance early completion, keep `completionCondition` inside the bounded subset: one boolean variable path like `approved` or `not approved`, or one counter comparison such as `completed >= 2`, `active == 0`, or `nrOfCompletedInstances >= 1`. Preserve workflow intent, but remove unsupported configuration '{detail}'."
            ),
            json!({
                "process_id": process_id,
                "node_id": node_id,
                "detail": detail,
            }),
        ),
        BpmnEngineError::UnsupportedTaskConfiguration {
            process_id,
            node_id,
            detail,
        } => return Some(task_configuration_issue(process_id, node_id, detail)),
        BpmnEngineError::UnknownCalledProcess {
            process_id,
            node_id,
            called_process_id,
        } => LintIssue::new(
            "bpmn.unknown_called_process",
            "Call activity targets a missing process",
            format!(
                "Process '{process_id}' call activity '{node_id}' references missing called process '{called_process_id}'."
            ),
            "The bounded engine can only enter a call activity when `calledElement` matches another executable process id in the same BPMN package.",
            vec![
                format!("Change `calledElement` on call activity '{node_id}' to an existing process id in the same BPMN package."),
                "If the child process is missing entirely, add that process definition before retrying parser validation.".to_string(),
            ],
            format!(
                "Edit process '{process_id}' so call activity '{node_id}' points its `calledElement` at an existing executable process in the same BPMN package. Preserve workflow intent, but do not leave it targeting missing process '{called_process_id}'."
            ),
            json!({
                "process_id": process_id,
                "node_id": node_id,
                "called_process_id": called_process_id,
            }),
        ),
        BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id,
            node_id,
            detail,
        } => return Some(subprocess_configuration_issue(process_id, node_id, detail)),
        BpmnEngineError::UnsupportedCompensationConfiguration {
            process_id,
            node_id,
            detail,
        } => return Some(compensation_configuration_issue(process_id, node_id, detail)),
        BpmnEngineError::UnsupportedGatewayConfiguration {
            process_id,
            node_id,
            detail,
        } => return Some(gateway_configuration_issue(process_id, node_id, detail)),
        BpmnEngineError::UnsupportedEventConfiguration {
            process_id,
            node_id,
            detail,
        } => LintIssue::new(
            "bpmn.unsupported_event_configuration",
            "Event configuration exceeds the bounded slice",
            format!(
                "Process '{process_id}' event node '{node_id}' uses unsupported configuration '{detail}'."
            ),
            "The current event slice supports one start event, intermediate catch event, exclusive event-based gateway wait target, or task-attached boundary event with exactly one conditional event definition and one bounded condition expression using a boolean variable path or numeric comparison, plus one escalation end event or intermediate escalation throw inside a bounded subprocess-like runtime scope when a matching interrupting escalation boundary exists on the parent owner.",
            vec![
                "Use one `conditionalEventDefinition` with one nested `condition` expression on a `startEvent` or `intermediateCatchEvent`, including an exclusive `eventBasedGateway` wait target, or attach that conditional definition to one supported interrupting or non-interrupting task `boundaryEvent`.".to_string(),
                "For escalation routing, place `escalationEventDefinition` on an end event or intermediate throw event inside a bounded embedded subprocess, same-package call activity, or transaction child scope and add a matching interrupting escalation boundary on the parent owner.".to_string(),
            ],
            format!(
                "Repair event node '{node_id}' in process '{process_id}' so it uses one bounded conditional expression, one bounded escalation end/throw-to-boundary route, or another supported event family. Preserve workflow intent, but remove unsupported configuration '{detail}'."
            ),
            json!({
                "process_id": process_id,
                "node_id": node_id,
                "detail": detail,
            }),
        ),
        BpmnEngineError::UnsupportedTransactionConfiguration {
            process_id,
            node_id,
            detail,
        } => return Some(transaction_configuration_issue(process_id, node_id, detail)),
        _ => return None,
    })
}
