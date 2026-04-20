//! BPMN lint entrypoint and error-to-guidance mapping.

use super::{LintDomain, LintIssue, LintReport};
use crate::error::BpmnEngineError;
use crate::parser::{BpmnParseOptions, BpmnSourceFile, parse_bpmn_package};
use serde_json::json;

/// Lints one BPMN source and returns an LLM-friendly blocking report.
#[must_use]
pub fn lint_bpmn_source(source: &BpmnSourceFile) -> LintReport {
    match parse_bpmn_package(std::slice::from_ref(source), &BpmnParseOptions::default()) {
        Ok(_) => LintReport::ok(LintDomain::Bpmn, &source.source_id),
        Err(error) => LintReport::blocking(
            LintDomain::Bpmn,
            &source.source_id,
            vec![issue_from_bpmn_error(source, &error)],
        ),
    }
}

fn issue_from_bpmn_error(source: &BpmnSourceFile, error: &BpmnEngineError) -> LintIssue {
    issue_from_bpmn_document_error(error)
        .or_else(|| issue_from_bpmn_identity_error(error))
        .or_else(|| issue_from_bpmn_reference_error(error))
        .or_else(|| issue_from_bpmn_topology_error(error))
        .or_else(|| issue_from_bpmn_execution_shape_error(error))
        .unwrap_or_else(|| unexpected_bpmn_issue(source, error))
}

fn issue_from_bpmn_document_error(error: &BpmnEngineError) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::InvalidXml { source_id, message } => LintIssue::new(
            "bpmn.invalid_xml",
            "BPMN XML is not well-formed",
            format!("Source '{source_id}' cannot be parsed as XML: {message}"),
            "The BPMN linter stops before workflow validation when the XML tree is malformed.",
            vec![
                "Repair the XML structure first: close open tags, fix broken attribute quotes, and remove overlapping elements.".to_string(),
                "Keep BPMN element names and ids stable while fixing XML syntax so workflow semantics do not drift.".to_string(),
            ],
            format!(
                "Repair the XML syntax in BPMN source '{source_id}' so it becomes well-formed without changing workflow intent. Preserve existing BPMN ids and task semantics while fixing broken tags, attribute quoting, or nesting."
            ),
            json!({
                "source_id": source_id,
                "parser_message": message,
            }),
        ),
        BpmnEngineError::MissingRootElement { source_id } => LintIssue::new(
            "bpmn.missing_root_element",
            "BPMN file has no root XML element",
            format!("Source '{source_id}' does not contain a root XML element."),
            "The linter cannot discover `<bpmn:definitions>` or any BPMN structure when the file is empty or structurally missing a root node.",
            vec![
                "Ensure the file contains one XML root element, usually `<bpmn:definitions>` for BPMN sources.".to_string(),
                "Place the BPMN process content under that root element instead of leaving the file empty or partially copied.".to_string(),
            ],
            format!(
                "Rewrite BPMN source '{source_id}' so it has one valid XML root element, typically `<bpmn:definitions>`, and move the workflow content under that root."
            ),
            json!({
                "source_id": source_id,
            }),
        ),
        BpmnEngineError::MissingAttribute {
            source_id,
            element,
            attribute,
        } => LintIssue::new(
            "bpmn.missing_attribute",
            "Required BPMN attribute is missing",
            format!(
                "Element '<{element}>' in source '{source_id}' is missing required attribute '{attribute}'."
            ),
            "The bounded BPMN parser requires this attribute to identify the node, process, or sequence-flow relationship.",
            vec![
                format!("Add the missing '{attribute}' attribute directly on the `<{element}>` element."),
                "Use a stable identifier or reference value that matches the surrounding BPMN structure.".to_string(),
            ],
            format!(
                "Edit BPMN source '{source_id}' and add the required '{attribute}' attribute to `<{element}>`. Reuse surrounding ids and references consistently so the workflow graph stays valid."
            ),
            json!({
                "source_id": source_id,
                "element": element,
                "attribute": attribute,
            }),
        ),
        BpmnEngineError::UnsupportedElement {
            source_id,
            process_id,
            element,
        } => LintIssue::new(
            "bpmn.unsupported_element",
            "BPMN element is outside the supported subset",
            format!(
                "Process '{process_id}' in source '{source_id}' uses unsupported element '<{element}>'."
            ),
            "The current engine only lints and parses a bounded BPMN subset, so unsupported elements block execution-oriented validation.",
            vec![
                format!("Replace `<{element}>` with an equivalent structure built from the supported subset when possible."),
                "If the workflow truly requires this element, preserve the original intent in comments or notes and defer execution until engine support exists.".to_string(),
            ],
            format!(
                "Edit BPMN source '{source_id}' so process '{process_id}' no longer uses unsupported element `<{element}>`. Preserve workflow intent, but rewrite the structure using only the supported bounded subset: startEvent, endEvent, intermediateCatchEvent with exactly one messageEventDefinition, signalEventDefinition, or timerEventDefinition, one interrupting timer boundaryEvent attached to one serviceTask/userTask/manualTask/businessRuleTask, one bounded callActivity that targets another executable process in the same BPMN package, serviceTask, userTask, manualTask, businessRuleTask, those same host-blocking task kinds with bounded `standardLoopCharacteristics`, those same host-blocking task kinds with bounded sequential `multiInstanceLoopCharacteristics isSequential=\"true\"` plus integer `loopCardinality`, exclusiveGateway, parallelGateway, one exclusive eventBasedGateway whose outgoing targets are message/signal/timer intermediateCatchEvent waits, and sequenceFlow. Do not introduce inclusiveGateway, embedded subProcess bodies, non-interrupting boundaries, parallel multi-instance expansion, multi-instance data input/output bindings, completionCondition, or full condition-driven routing in this bounded slice."
            ),
            json!({
                "source_id": source_id,
                "process_id": process_id,
                "element": element,
            }),
        ),
        BpmnEngineError::MissingProcessDefinitions { source_id } => LintIssue::new(
            "bpmn.missing_process_definitions",
            "BPMN file contains no process definitions",
            format!("Source '{source_id}' does not contain any `<process>` definitions."),
            "The engine cannot lint or execute BPMN content if the file has only wrapper metadata and no workflow process.",
            vec![
                "Add at least one `<bpmn:process>` element under the BPMN definitions root.".to_string(),
                "Move task, event, and sequence-flow elements inside that process instead of leaving them at the wrong XML level.".to_string(),
            ],
            format!(
                "Rewrite BPMN source '{source_id}' so it contains at least one `<bpmn:process>` definition with the workflow nodes and sequence flows nested inside it."
            ),
            json!({
                "source_id": source_id,
            }),
        ),
        _ => return None,
    })
}

fn issue_from_bpmn_identity_error(error: &BpmnEngineError) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::MissingRequiredNodeElement {
            process_id,
            node_id,
            element,
        } => LintIssue::new(
            "bpmn.missing_required_node_element",
            "Required BPMN node structure is missing",
            format!("Process '{process_id}' node '{node_id}' is missing required element '{element}'."),
            "The bounded parser requires this node-level child structure before it can materialize the BPMN wait or routing semantics.",
            vec![
                format!("Add the missing '{element}' child structure directly under BPMN node '{node_id}'."),
                "Keep the surrounding node id and sequence-flow references stable while repairing the missing node internals.".to_string(),
            ],
            format!(
                "Edit process '{process_id}' so BPMN node '{node_id}' includes the required '{element}' child structure. Preserve the existing node id and surrounding sequence flows while repairing the missing event or node internals."
            ),
            json!({
                "process_id": process_id,
                "node_id": node_id,
                "element": element,
            }),
        ),
        BpmnEngineError::MissingRequiredProcessElement {
            process_id,
            element,
        } => LintIssue::new(
            "bpmn.missing_required_process_element",
            "Required BPMN process element is missing",
            format!("Process '{process_id}' is missing required element '{element}'."),
            "The bounded runtime expects a complete start-to-end process shape before it can validate flow structure.",
            vec![
                "Add the missing required process element before adjusting downstream flows.".to_string(),
                "Ensure sequence flows connect to the new element with consistent ids and references.".to_string(),
            ],
            format!(
                "Repair process '{process_id}' by adding the missing required element '{element}' and then reconnect sequence flows so the process has a valid start-to-end structure."
            ),
            json!({
                "process_id": process_id,
                "element": element,
            }),
        ),
        _ => return None,
    })
}

fn issue_from_bpmn_reference_error(error: &BpmnEngineError) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::DuplicateProcessId {
            package_id,
            process_id,
        } => LintIssue::new(
            "bpmn.duplicate_process_id",
            "Duplicate BPMN process id",
            format!(
                "Package '{package_id}' defines process id '{process_id}' more than once."
            ),
            "Process ids must be unique so the engine can resolve one stable execution target.",
            vec![
                "Rename one of the duplicate processes to a unique id.".to_string(),
                "Update any references or adapter configuration that point to the renamed process.".to_string(),
            ],
            format!(
                "Edit the BPMN package so process id '{process_id}' is unique within package '{package_id}'. Rename duplicates and keep downstream references consistent."
            ),
            json!({
                "package_id": package_id,
                "process_id": process_id,
            }),
        ),
        BpmnEngineError::DuplicateNodeId { process_id, node_id } => LintIssue::new(
            "bpmn.duplicate_node_id",
            "Duplicate BPMN node id",
            format!("Process '{process_id}' defines node id '{node_id}' more than once."),
            "Node ids must be unique so sequence flows and runtime state can point to one unambiguous BPMN node.",
            vec![
                "Rename one of the duplicate node ids to a unique value within the process.".to_string(),
                "Update any sequenceFlow sourceRef or targetRef values that should point to the renamed node.".to_string(),
            ],
            format!(
                "Edit process '{process_id}' so node id '{node_id}' becomes unique. If you rename a node, also update all sequenceFlow sourceRef and targetRef references that should follow it."
            ),
            json!({
                "process_id": process_id,
                "node_id": node_id,
            }),
        ),
        BpmnEngineError::DuplicateSequenceFlowId { process_id, flow_id } => LintIssue::new(
            "bpmn.duplicate_sequence_flow_id",
            "Duplicate sequence flow id",
            format!(
                "Process '{process_id}' defines sequence flow id '{flow_id}' more than once."
            ),
            "Sequence flow ids must be unique so diagnostics and graph normalization can identify one edge at a time.",
            vec![
                "Rename one of the duplicate sequence flows to a unique id.".to_string(),
                "Keep sourceRef and targetRef unchanged unless the edge meaning also needs to change.".to_string(),
            ],
            format!(
                "Edit process '{process_id}' so sequence flow id '{flow_id}' is unique. Rename only the conflicting flow ids unless the edge semantics also need correction."
            ),
            json!({
                "process_id": process_id,
                "flow_id": flow_id,
            }),
        ),
        BpmnEngineError::UnknownSequenceFlowEndpoint {
            process_id,
            flow_id,
            endpoint,
            node_id,
        } => LintIssue::new(
            "bpmn.unknown_sequence_flow_endpoint",
            "Sequence flow points to an unknown node",
            format!(
                "Process '{process_id}' sequence flow '{flow_id}' references unknown {endpoint} node '{node_id}'."
            ),
            "The graph cannot be normalized when a sequence flow points to a node id that does not exist in the process.",
            vec![
                format!("Either create node '{node_id}' or change the {endpoint}Ref on flow '{flow_id}' to an existing node id."),
                "Re-check both ends of the flow after the fix so the process remains connected.".to_string(),
            ],
            format!(
                "Edit process '{process_id}' so sequence flow '{flow_id}' no longer references missing {endpoint} node '{node_id}'. Either add the missing node or retarget the flow to an existing node id."
            ),
            json!({
                "process_id": process_id,
                "flow_id": flow_id,
                "endpoint": endpoint,
                "node_id": node_id,
            }),
        ),
        _ => return None,
    })
}

fn issue_from_bpmn_topology_error(error: &BpmnEngineError) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::UnknownBoundaryAttachment {
            process_id,
            node_id,
            attached_to_node_id,
        } => LintIssue::new(
            "bpmn.unknown_boundary_attachment",
            "Boundary event attaches to an unknown node",
            format!(
                "Process '{process_id}' boundary event '{node_id}' references missing attached node '{attached_to_node_id}'."
            ),
            "The bounded engine can only normalize a boundary event when `attachedToRef` points to an existing host-blocking task node in the same process.",
            vec![
                format!("Change `attachedToRef` on boundary event '{node_id}' to an existing node id in process '{process_id}'."),
                "Attach the boundary only to one serviceTask, userTask, manualTask, or businessRuleTask.".to_string(),
            ],
            format!(
                "Edit process '{process_id}' so boundary event '{node_id}' uses an `attachedToRef` that points to an existing serviceTask, userTask, manualTask, or businessRuleTask. Preserve workflow intent, but do not leave the boundary attached to missing node '{attached_to_node_id}'."
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
        } => LintIssue::new(
            "bpmn.unsupported_boundary_configuration",
            "Boundary event configuration exceeds the bounded slice",
            format!(
                "Process '{process_id}' boundary event '{node_id}' uses unsupported configuration '{detail}'."
            ),
            "The current engine supports only one interrupting timer boundary event attached to one host-blocking task.",
            vec![
                "Keep the timeout or escalation intent, but rewrite the boundary as one interrupting timer boundary event.".to_string(),
                "Attach it to one serviceTask, userTask, manualTask, or businessRuleTask and use `cancelActivity=\"true\"`.".to_string(),
            ],
            format!(
                "Rewrite boundary event '{node_id}' in process '{process_id}' so it fits the bounded slice: one interrupting timer `boundaryEvent` attached to one serviceTask, userTask, manualTask, or businessRuleTask with `cancelActivity=\"true\"` and exactly one timer expression. Preserve workflow intent, but remove unsupported configuration '{detail}'."
            ),
            json!({
                "process_id": process_id,
                "node_id": node_id,
                "detail": detail,
            }),
        ),
        BpmnEngineError::UnsupportedEventBasedGatewayConfiguration {
            process_id,
            node_id,
            detail,
        } => LintIssue::new(
            "bpmn.unsupported_event_based_gateway_configuration",
            "Event-based gateway configuration exceeds the bounded slice",
            format!(
                "Process '{process_id}' event-based gateway '{node_id}' uses unsupported configuration '{detail}'."
            ),
            "The current engine supports only one bounded event-based gateway shape: one exclusive eventBasedGateway whose outgoing paths all target message, signal, or timer intermediate catch events.",
            vec![
                "Keep the winner-takes-all intent, but make every outgoing branch from the eventBasedGateway point to one intermediateCatchEvent.".to_string(),
                "Use only messageEventDefinition, signalEventDefinition, or timerEventDefinition on those waiting nodes in this bounded slice.".to_string(),
            ],
            format!(
                "Rewrite event-based gateway '{node_id}' in process '{process_id}' so it fits the bounded slice: use one exclusive `eventBasedGateway` with at least two outgoing branches, and make every outgoing target one `intermediateCatchEvent` with exactly one `messageEventDefinition`, `signalEventDefinition`, or `timerEventDefinition`. Preserve workflow intent, but remove unsupported configuration '{detail}'."
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

fn issue_from_bpmn_execution_shape_error(error: &BpmnEngineError) -> Option<LintIssue> {
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
            "The current engine supports two bounded repeatable-task shapes: `standardLoopCharacteristics` on one serviceTask, userTask, manualTask, or businessRuleTask with a positive `loopMaximum` or one simple boolean loop condition, and sequential `multiInstanceLoopCharacteristics isSequential=\"true\"` on those same host-blocking task kinds with an integer `loopCardinality`. Parallel multi-instance expansion and richer data bindings remain deferred.",
            vec![
                "If you need bounded repeat execution now, rewrite the node to one serviceTask, userTask, manualTask, or businessRuleTask with either `standardLoopCharacteristics` or sequential `multiInstanceLoopCharacteristics isSequential=\"true\"`.".to_string(),
                "Use a positive `loopMaximum`, or one simple boolean condition like `done` or `not done`, or one integer `loopCardinality`. Keep parallel multi-instance, data input/output bindings, and `completionCondition` out of this bounded slice.".to_string(),
            ],
            format!(
                "Rewrite loop node '{node_id}' in process '{process_id}' so it fits the bounded slice: either use one `standardLoopCharacteristics` block on a serviceTask, userTask, manualTask, or businessRuleTask with a positive `loopMaximum` or one simple boolean loop condition like `done` or `not done`, or use one sequential `multiInstanceLoopCharacteristics` block with `isSequential=\"true\"` and an integer `loopCardinality` on those same host-blocking task kinds. Preserve workflow intent, but remove unsupported configuration '{detail}', and do not introduce parallel multi-instance, data input/output bindings, or `completionCondition`."
            ),
            json!({
                "process_id": process_id,
                "node_id": node_id,
                "detail": detail,
            }),
        ),
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
        } => LintIssue::new(
            "bpmn.unsupported_subprocess_configuration",
            "Subprocess configuration exceeds the bounded slice",
            format!(
                "Process '{process_id}' subprocess node '{node_id}' uses unsupported configuration '{detail}'."
            ),
            "The current engine supports only one bounded callActivity shape that targets another process in the same BPMN package; embedded subprocess bodies and recursive nesting remain deferred.",
            vec![
                "Keep the nested workflow intent, but rewrite it as one non-recursive callActivity that targets another executable process in the same BPMN package.".to_string(),
                "Do not introduce embedded subProcess bodies or recursive call chains in this bounded slice.".to_string(),
            ],
            format!(
                "Rewrite subprocess node '{node_id}' in process '{process_id}' so it fits the bounded slice: use one non-recursive `callActivity` with a valid `calledElement` that points to another executable process in the same BPMN package. Preserve workflow intent, but remove unsupported configuration '{detail}'."
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

fn unexpected_bpmn_issue(source: &BpmnSourceFile, error: &BpmnEngineError) -> LintIssue {
    LintIssue::new(
        "bpmn.unexpected_engine_error",
        "Unexpected BPMN lint error",
        format!(
            "BPMN linting for source '{}' returned unexpected engine error: {error}",
            source.source_id
        ),
        "The linter expected a parse or validation error but received a broader engine error, which usually indicates a missing lint mapping.",
        vec![
            "Inspect the source and the emitted evidence before making broad edits.".to_string(),
            "If the source is valid, extend the linter mapping instead of forcing a speculative workflow rewrite.".to_string(),
        ],
        format!(
            "Do not rewrite BPMN source '{}' blindly. First inspect the unexpected engine error and repair only the concrete structure proven by the evidence.",
            source.source_id
        ),
        json!({
            "source_id": source.source_id,
            "engine_error": error.to_string(),
        }),
    )
}
