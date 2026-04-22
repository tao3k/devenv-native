use crate::error::BpmnEngineError;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn issue_from_bpmn_document_error(error: &BpmnEngineError) -> Option<LintIssue> {
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
            "Edit BPMN source '{source_id}' so process '{process_id}' no longer uses unsupported element `<{element}>`. Preserve workflow intent, but rewrite the structure using only the supported bounded subset: startEvent, endEvent, intermediateCatchEvent with exactly one messageEventDefinition, signalEventDefinition, or timerEventDefinition, receiveTask with exactly one message binding through `messageRef` or one nested `messageEventDefinition`, sendTask with exactly one message binding through `messageRef` or one nested `messageEventDefinition`, one interrupting timer boundaryEvent attached to one serviceTask/userTask/manualTask/businessRuleTask, one interrupting cancel boundaryEvent attached to one bounded `<transaction>` shell, one or more interrupting error boundaryEvent nodes attached to one bounded `<transaction>` shell, one bounded transaction cancel end path with exactly one nested `cancelEventDefinition` end event plus the matching parent cancel boundary, one bounded transaction error end path with exactly one nested `errorEventDefinition` end event plus every matching parent error boundary on that same transaction owner whose optional `errorRef` either matches the thrown error or stays omitted as a catch-all, one bounded compensation binding inside one bounded `<transaction>` shell using one compensation boundary attached to one completed host-blocking activity plus one detached `isForCompensation=\"true\"` host-blocking handler activity reached through one association, one synchronous throw-compensation end event inside one bounded `<transaction>` shell that either uses explicit `activityRef` to target one already compensable activity or omits `activityRef` to replay every already compensable activity in reverse completion order, one synchronous throw-compensation intermediate event inside one bounded `<transaction>` shell with explicit `activityRef` targeting one already compensable activity in that same shell before normal sequence-flow routing resumes, one bounded embedded `subProcess` body with exactly one nested `startEvent` and at least one nested `endEvent`, one bounded `<transaction>` shell with exactly one nested `startEvent` and at least one nested `endEvent`, one bounded callActivity that targets another executable process in the same BPMN package, serviceTask, userTask, manualTask, businessRuleTask, those same host-blocking task kinds with bounded `standardLoopCharacteristics`, those same host-blocking task kinds with bounded `multiInstanceLoopCharacteristics` in sequential (`isSequential=\"true\"`) or bounded parallel (omitted or `isSequential=\"false\"`) mode using either integer `loopCardinality` or one collection binding with `loopDataInputRef` plus `inputDataItem`, optional paired `loopDataOutputRef` plus `outputDataItem`, and bounded `completionCondition`, exclusiveGateway with simple boolean-path or numeric-comparison `sequenceFlow` `conditionExpression` values plus one optional `default` flow, structured inclusiveGateway with the same bounded condition/default subset plus one linear matching join fragment, parallelGateway, one exclusive eventBasedGateway whose outgoing targets are message/signal/timer intermediateCatchEvent waits, and sequenceFlow. Do not introduce scriptTask execution, correlations, broader collaboration-aware message routing, broader unstructured inclusive joins, non-interrupting boundaries, timer boundaries on transaction shells, cancel or error ends outside one bounded transaction shell, more than one cancel boundary on the same transaction owner, compensation event subprocesses, asynchronous or default throw-compensation intermediate events, asynchronous or broader throw-compensation end-event forms, non-transaction multi-boundary ownership, in-place multi-instance output bindings where `loopDataOutputRef` equals `loopDataInputRef`, or broader FEEL/script-backed gateway conditions in this bounded slice."
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
