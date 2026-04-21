//! BPMN lint entrypoint and error-to-guidance mapping.

use crate::bpmn_parse_api::{BpmnParseOptions, BpmnSourceFile, parse_bpmn_package};
use crate::error::BpmnEngineError;
use crate::lint_api::{LintDomain, LintIssue, LintReport};
use serde_json::json;

/// Lints one BPMN source and returns an LLM-friendly blocking report.
#[must_use]
pub(crate) fn lint_bpmn_source_impl(source: &BpmnSourceFile) -> LintReport {
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
        BpmnEngineError::InvalidXml { source_id, message } => invalid_xml_issue(source_id, message),
        BpmnEngineError::MissingRootElement { source_id } => missing_root_element_issue(source_id),
        BpmnEngineError::MissingAttribute {
            source_id,
            element,
            attribute,
        } => missing_attribute_issue(source_id, element, attribute),
        BpmnEngineError::UnsupportedElement {
            source_id,
            process_id,
            element,
        } => unsupported_element_issue(source_id, process_id, element),
        BpmnEngineError::MissingProcessDefinitions { source_id } => {
            missing_process_definitions_issue(source_id)
        }
        _ => return None,
    })
}

fn invalid_xml_issue(source_id: &str, message: &str) -> LintIssue {
    LintIssue::new(
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
    )
}

fn missing_root_element_issue(source_id: &str) -> LintIssue {
    LintIssue::new(
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
    )
}

fn missing_attribute_issue(source_id: &str, element: &str, attribute: &str) -> LintIssue {
    LintIssue::new(
        "bpmn.missing_attribute",
        "Required BPMN attribute is missing",
        format!("Element '<{element}>' in source '{source_id}' is missing required attribute '{attribute}'."),
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
    )
}

fn unsupported_element_issue(source_id: &str, process_id: &str, element: &str) -> LintIssue {
    LintIssue::new(
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
            "Edit BPMN source '{source_id}' so process '{process_id}' no longer uses unsupported element `<{element}>`. Preserve workflow intent, but rewrite the structure using only the supported bounded subset: startEvent, endEvent, intermediateCatchEvent with exactly one messageEventDefinition, signalEventDefinition, or timerEventDefinition, one interrupting timer boundaryEvent attached to one serviceTask/userTask/manualTask/businessRuleTask, one interrupting cancel boundaryEvent attached to one bounded `<transaction>` shell, one or more interrupting error boundaryEvent nodes attached to one bounded `<transaction>` shell, one bounded transaction cancel end path with exactly one nested `cancelEventDefinition` end event plus the matching parent cancel boundary, one bounded transaction error end path with exactly one nested `errorEventDefinition` end event plus every matching parent error boundary on that same transaction owner whose optional `errorRef` either matches the thrown error or stays omitted as a catch-all, one bounded compensation binding inside one bounded `<transaction>` shell using one compensation boundary attached to one completed host-blocking activity plus one detached `isForCompensation=\"true\"` host-blocking handler activity reached through one association, one bounded embedded `subProcess` body with exactly one nested `startEvent` and at least one nested `endEvent`, one bounded `<transaction>` shell with exactly one nested `startEvent` and at least one nested `endEvent`, one bounded callActivity that targets another executable process in the same BPMN package, serviceTask, userTask, manualTask, businessRuleTask, those same host-blocking task kinds with bounded `standardLoopCharacteristics`, those same host-blocking task kinds with bounded `multiInstanceLoopCharacteristics` in sequential (`isSequential=\"true\"`) or bounded parallel (omitted or `isSequential=\"false\"`) mode using either integer `loopCardinality` or one collection binding with `loopDataInputRef` plus `inputDataItem`, optional paired `loopDataOutputRef` plus `outputDataItem`, and bounded `completionCondition`, exclusiveGateway, parallelGateway, one exclusive eventBasedGateway whose outgoing targets are message/signal/timer intermediateCatchEvent waits, and sequenceFlow. Do not introduce inclusiveGateway, non-interrupting boundaries, timer boundaries on transaction shells, cancel or error ends outside one bounded transaction shell, more than one cancel boundary on the same transaction owner, throw compensation events, compensation event subprocesses, default compensation, non-transaction multi-boundary ownership, in-place multi-instance output bindings where `loopDataOutputRef` equals `loopDataInputRef`, or full condition-driven routing in this bounded slice."
        ),
        json!({
            "source_id": source_id,
            "process_id": process_id,
            "element": element,
        }),
    )
}

fn missing_process_definitions_issue(source_id: &str) -> LintIssue {
    LintIssue::new(
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
    )
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
        } => return Some(boundary_configuration_issue(process_id, node_id, detail)),
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
            "The current engine supports three bounded repeatable-task shapes: `standardLoopCharacteristics` on one serviceTask, userTask, manualTask, or businessRuleTask with a positive `loopMaximum` or one simple boolean loop condition, sequential `multiInstanceLoopCharacteristics isSequential=\"true\"` on those same host-blocking task kinds with either integer `loopCardinality` or collection-backed `loopDataInputRef` plus `inputDataItem`, and bounded parallel `multiInstanceLoopCharacteristics` with omitted or `isSequential=\"false\"` using that same bounded cardinality-or-collection expansion. Those multi-instance shapes may also carry one bounded optional output aggregation pair `loopDataOutputRef` plus `outputDataItem`, as long as the output path is different from the input path, and one bounded `completionCondition` using either one simple boolean variable path such as `approved` or `not approved`, or one counter comparison using `completed`, `active`, or `total` and their BPMN aliases `nrOfCompletedInstances`, `nrOfActiveInstances`, and `nrOfInstances`.",
            vec![
                "If you need bounded repeat execution now, rewrite the node to one serviceTask, userTask, manualTask, or businessRuleTask with either `standardLoopCharacteristics`, sequential `multiInstanceLoopCharacteristics isSequential=\"true\"`, or bounded parallel `multiInstanceLoopCharacteristics` with omitted or `isSequential=\"false\"`.".to_string(),
                "Use a positive `loopMaximum`, or one simple boolean condition like `done` or `not done`, or for multi-instance use either integer `loopCardinality` or one collection binding with `loopDataInputRef` plus `inputDataItem`. If you aggregate per-iteration output, provide both `loopDataOutputRef` and `outputDataItem`, keep the output path different from the input path, and for early completion keep `completionCondition` inside the bounded subset with one boolean variable path or one counter comparison like `completed >= 2`.".to_string(),
            ],
            format!(
                "Rewrite loop node '{node_id}' in process '{process_id}' so it fits the bounded slice: either use one `standardLoopCharacteristics` block on a serviceTask, userTask, manualTask, or businessRuleTask with a positive `loopMaximum` or one simple boolean loop condition like `done` or `not done`, or use one `multiInstanceLoopCharacteristics` block on those same host-blocking task kinds, setting `isSequential=\"true\"` for sequential execution or leaving it omitted or `isSequential=\"false\"` for bounded parallel execution, and choosing exactly one expansion mode: integer `loopCardinality`, or collection-backed `loopDataInputRef` plus `inputDataItem`. If you aggregate iteration output, keep `loopDataOutputRef` and `outputDataItem` paired and make the output path different from the input path. If you need multi-instance early completion, keep `completionCondition` inside the bounded subset: one boolean variable path like `approved` or `not approved`, or one counter comparison such as `completed >= 2`, `active == 0`, or `nrOfCompletedInstances >= 1`. Preserve workflow intent, but remove unsupported configuration '{detail}'."
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
        } => return Some(subprocess_configuration_issue(process_id, node_id, detail)),
        BpmnEngineError::UnsupportedCompensationConfiguration {
            process_id,
            node_id,
            detail,
        } => return Some(compensation_configuration_issue(process_id, node_id, detail)),
        BpmnEngineError::UnsupportedTransactionConfiguration {
            process_id,
            node_id,
            detail,
        } => return Some(transaction_configuration_issue(process_id, node_id, detail)),
        _ => return None,
    })
}

fn boundary_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    match detail {
        "cancel_boundary_requires_transaction_shell" => {
            cancel_boundary_requires_transaction_shell_issue(process_id, node_id, detail)
        }
        "multiple_transaction_cancel_boundaries" => {
            multiple_transaction_cancel_boundaries_issue(process_id, node_id, detail)
        }
        "error_boundary_requires_transaction_shell" => {
            error_boundary_requires_transaction_shell_issue(process_id, node_id, detail)
        }
        _ => generic_boundary_configuration_issue(process_id, node_id, detail),
    }
}

fn cancel_boundary_requires_transaction_shell_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_boundary_configuration",
        "Cancel boundary must attach to a transaction shell",
        format!(
            "Process '{process_id}' boundary event '{node_id}' uses `<cancelEventDefinition>` without attaching it to a bounded transaction shell."
        ),
        "The bounded engine supports one interrupting cancel boundary path only when the boundary event is attached to one bounded `<transaction>` shell and matches the transaction shell's nested cancel end.",
        vec![
            "Attach this cancel boundary to one `<bpmn:transaction>` node, not to a task, embedded subprocess, or call activity.".to_string(),
            "Keep `cancelActivity=\"true\"` and pair the boundary with exactly one nested transaction end event that carries `<bpmn:cancelEventDefinition>`.".to_string(),
        ],
        format!(
            "Rewrite boundary event '{node_id}' in process '{process_id}' so `<cancelEventDefinition>` is used only as one interrupting boundary event attached to one bounded `<bpmn:transaction>` shell, paired with exactly one nested cancel end inside that same transaction. Preserve workflow intent, but do not leave the cancel boundary attached to a task or non-transaction subprocess."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn multiple_transaction_cancel_boundaries_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_boundary_configuration",
        "Transaction owner exposes more than one cancel boundary",
        format!(
            "Process '{process_id}' boundary event '{node_id}' adds a second `<cancelEventDefinition>` boundary to the same bounded transaction owner."
        ),
        "The bounded engine allows one transaction owner to expose one interrupting cancel boundary plus one or more interrupting error boundaries, but it still permits only one cancel boundary on that same transaction shell.",
        vec![
            "Keep exactly one interrupting cancel boundary attached to this `<bpmn:transaction>` node.".to_string(),
            "If the second branch is really an error path, rewrite it as `<bpmn:errorEventDefinition>` and pair it with the transaction shell's nested error end.".to_string(),
        ],
        format!(
            "Repair process '{process_id}' so boundary event '{node_id}' no longer creates a second transaction cancel boundary. Keep one interrupting `<bpmn:cancelEventDefinition>` boundary on that transaction owner, and convert any extra recovery branch into one or more interrupting `<bpmn:errorEventDefinition>` boundaries only if the nested transaction end throws matching errors."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn error_boundary_requires_transaction_shell_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_boundary_configuration",
        "Error boundary must attach to a transaction shell",
        format!(
            "Process '{process_id}' boundary event '{node_id}' uses `<errorEventDefinition>` without attaching it to a bounded transaction shell."
        ),
        "The bounded engine supports one or more interrupting error boundary paths only when those boundary events are attached to one bounded `<transaction>` shell and match that transaction shell's nested error end.",
        vec![
            "Attach this error boundary to one `<bpmn:transaction>` node, not to a task, embedded subprocess, or call activity.".to_string(),
            "Keep `cancelActivity=\"true\"` and pair the boundary with exactly one nested transaction end event that carries `<bpmn:errorEventDefinition>`, using a matching `errorRef` or omitting `errorRef` on the boundary as a catch-all.".to_string(),
        ],
        format!(
            "Rewrite boundary event '{node_id}' in process '{process_id}' so `<errorEventDefinition>` is used only as one interrupting boundary event attached to one bounded `<bpmn:transaction>` shell, paired with exactly one nested error end inside that same transaction. Preserve workflow intent, but do not leave the error boundary attached to a task or non-transaction subprocess."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn generic_boundary_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_boundary_configuration",
        "Boundary event configuration exceeds the bounded slice",
        format!(
            "Process '{process_id}' boundary event '{node_id}' uses unsupported configuration '{detail}'."
        ),
        "The current engine supports only three interrupting boundary ownership shapes: one timer boundary attached to one host-blocking task, one cancel boundary attached to one bounded transaction shell, or one or more error boundaries attached to one bounded transaction shell.",
        vec![
            "Keep the timeout, escalation, or transaction-cancel intent, but rewrite the boundary to one supported interrupting shape.".to_string(),
            "Use a timer boundary on one serviceTask, userTask, manualTask, or businessRuleTask, or use one cancel boundary plus one or more error boundaries on one bounded `<transaction>` shell.".to_string(),
        ],
        format!(
            "Rewrite boundary event '{node_id}' in process '{process_id}' so it fits the bounded slice: either one interrupting timer `boundaryEvent` attached to one serviceTask, userTask, manualTask, or businessRuleTask with `cancelActivity=\"true\"` and exactly one timer expression, one interrupting cancel `boundaryEvent` attached to one bounded `<bpmn:transaction>` shell with a matching nested cancel end, or one or more interrupting error `boundaryEvent` nodes attached to one bounded `<bpmn:transaction>` shell with a matching nested error end whose optional `errorRef` either matches the thrown error or stays omitted as a catch-all. Preserve workflow intent, but remove unsupported configuration '{detail}'."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn subprocess_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    match detail {
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

fn transaction_configuration_issue(
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
        "multiple_transaction_error_end_events" => {
            multiple_transaction_error_end_issue(process_id, node_id, detail)
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

fn error_end_requires_transaction_shell_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_transaction_configuration",
        "Error end event must live inside a transaction shell",
        format!(
            "Process '{process_id}' end event '{node_id}' uses `<errorEventDefinition>` outside one bounded transaction shell."
        ),
        "The bounded engine supports a transaction error end only as part of one transaction-error path: the end event must live inside one nested `<bpmn:transaction>` shell and be paired with one parent interrupting error boundary attached to that same transaction node.",
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
    LintIssue::new(
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
    LintIssue::new(
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

fn multiple_transaction_error_end_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_transaction_configuration",
        "Transaction shell supports only one error end event",
        format!(
            "Process '{process_id}' transaction node '{node_id}' contains more than one nested error end event."
        ),
        "The bounded transaction-error slice supports exactly one nested `<bpmn:endEvent>` carrying `<bpmn:errorEventDefinition>` inside one transaction shell, so the engine can map that path to one parent interrupting error boundary deterministically.",
        vec![
            "Keep at most one nested error end inside this `<bpmn:transaction>` body.".to_string(),
            "If multiple error outcomes are needed, merge them through internal gateways and route them into one shared bounded error end.".to_string(),
        ],
        format!(
            "Repair transaction node '{node_id}' in process '{process_id}' so its bounded `<bpmn:transaction>` body contains exactly one nested error end event with `<bpmn:errorEventDefinition>`. Preserve workflow intent, but merge multiple error exits into one bounded error path."
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
    LintIssue::new(
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
    LintIssue::new(
        "bpmn.unsupported_transaction_configuration",
        "Transaction error path is missing the parent error boundary",
        format!(
            "Process '{process_id}' transaction node '{node_id}' contains a nested error end but does not expose any matching parent interrupting error boundary."
        ),
        "The bounded engine only executes transaction error semantics when one transaction shell has both sides of the path: one nested error end inside the child body and one or more matching parent interrupting error boundaries attached to that same transaction node. If a boundary carries `errorRef`, it must match the thrown error; if it omits `errorRef`, it acts as the bounded catch-all path.",
        vec![
            "Add one or more interrupting `boundaryEvent` nodes with `<bpmn:errorEventDefinition>` attached to this `<bpmn:transaction>` node.".to_string(),
            "If the nested error end uses `errorRef`, either copy that same `errorRef` to one or more matching boundaries or omit `errorRef` on one boundary to make it the bounded catch-all path.".to_string(),
        ],
        format!(
            "Repair transaction node '{node_id}' in process '{process_id}' so its bounded error path is complete: keep exactly one nested error end inside the `<bpmn:transaction>` body and add one or more parent interrupting `boundaryEvent` nodes with `<bpmn:errorEventDefinition>` attached to that same transaction node, routing the error path through each selected boundary's outgoing sequence flow. If the nested error end declares `errorRef`, make one or more boundaries use the same `errorRef` or omit `errorRef` on one boundary to catch that error generically."
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

fn generic_transaction_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    LintIssue::new(
        "bpmn.unsupported_transaction_configuration",
        "Transaction configuration exceeds the bounded slice",
        format!(
            "Process '{process_id}' transaction node '{node_id}' uses unsupported configuration '{detail}'."
        ),
        "The current engine supports only one bounded transaction shell shape: exactly one nested start event, at least one nested end event, and at most one bounded cancel path or one bounded error path, each paired with one matching parent interrupting boundary event.",
        vec![
            "Keep the transaction intent, but reduce it to the bounded transaction shell shape.".to_string(),
            "If the model depends on richer BPMN transaction features such as throw compensation events, compensation event subprocesses, or default compensation, preserve that requirement explicitly and defer execution until support lands.".to_string(),
        ],
        format!(
            "Rewrite transaction node '{node_id}' in process '{process_id}' so it fits the bounded slice: one `<bpmn:transaction>` shell with exactly one nested `<bpmn:startEvent>`, at least one nested `<bpmn:endEvent>`, and at most one bounded cancel path or one bounded error path, each composed of one nested throwing end event plus one matching parent interrupting boundary on that transaction owner. Preserve workflow intent, but remove unsupported configuration '{detail}'."
        ),
        json!({
            "process_id": process_id,
            "node_id": node_id,
            "detail": detail,
        }),
    )
}

fn compensation_configuration_issue(
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> LintIssue {
    match detail {
        "compensation_requires_transaction_shell" => LintIssue::new(
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
        ),
        _ => LintIssue::new(
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
        ),
    }
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
