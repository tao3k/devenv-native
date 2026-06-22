use crate::error::BpmnEngineError;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn issue_from_bpmn_document_error(error: &BpmnEngineError) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::InvalidXml {
            source_id,
            message,
            offset,
        } => invalid_xml_issue(source_id, message, *offset),
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

fn invalid_xml_issue(source_id: &str, message: &str, offset: Option<u64>) -> LintIssue {
    LintIssue::from_parts(
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
            "parser_offset": offset,
        }),
    )
}

fn missing_root_element_issue(source_id: &str) -> LintIssue {
    LintIssue::from_parts(
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
    LintIssue::from_parts(
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
    if element == "complexGateway" {
        return unsupported_complex_gateway_issue(source_id, process_id);
    }

    LintIssue::from_parts(
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
            "Edit BPMN source '{source_id}' so process '{process_id}' no longer uses unsupported element `<{element}>`. Preserve workflow intent, but rewrite the structure using only the supported bounded subset: startEvent with at most one messageEventDefinition, signalEventDefinition, timerEventDefinition, or conditionalEventDefinition with one bounded condition, endEvent, intermediateCatchEvent with exactly one messageEventDefinition, signalEventDefinition, timerEventDefinition, or conditionalEventDefinition with one bounded condition, receiveTask with exactly one message binding through `messageRef` or one nested `messageEventDefinition`, sendTask with exactly one message binding through `messageRef` or one nested `messageEventDefinition`, scriptTask with one optional `scriptFormat` and one optional nested `<bpmn:script>` body, one interrupting timer boundaryEvent attached to one serviceTask/scriptTask/userTask/manualTask/businessRuleTask, one interrupting message, signal, or conditional boundaryEvent attached to one serviceTask/scriptTask/userTask/manualTask/businessRuleTask, one interrupting timer, message, signal, or conditional boundaryEvent attached to one bounded embedded `<bpmn:subProcess>` owner, including the bounded mixed-owner shape with that one interrupting timer/message/signal/conditional boundary plus one or more interrupting error boundaryEvent nodes on the same owner, one interrupting timer, message, signal, or conditional boundaryEvent attached to one bounded same-package `<bpmn:callActivity>` owner, including the bounded mixed-owner shape with that one interrupting timer/message/signal/conditional boundary plus one or more interrupting error boundaryEvent nodes on the same owner, one interrupting timer, message, signal, or conditional boundaryEvent attached to one bounded `<transaction>` shell either on its own, including the bounded mixed-owner shape with that one interrupting timer/message/signal/conditional boundary plus one interrupting cancel boundary on the same owner, including the bounded mixed-owner shape with that one interrupting timer/message/signal/conditional boundary plus one or more interrupting error boundaryEvent nodes on the same owner, or including the bounded mixed-owner shape with that one interrupting timer/message/signal/conditional boundary plus one interrupting cancel boundary and one or more interrupting error boundaryEvent nodes on the same owner, one non-interrupting timer boundaryEvent attached to one non-repeating, bounded `standardLoopCharacteristics`, or bounded sequential or parallel multi-instance serviceTask/scriptTask/userTask/manualTask/businessRuleTask, one non-interrupting message, signal, or conditional boundaryEvent attached to one non-repeating, bounded `standardLoopCharacteristics`, or bounded sequential or parallel multi-instance serviceTask/scriptTask/userTask/manualTask/businessRuleTask, one interrupting cancel boundaryEvent attached to one bounded `<transaction>` shell, one or more interrupting error boundaryEvent nodes attached to one bounded `<transaction>` shell, one bounded embedded `<bpmn:subProcess>` shell, or one bounded same-package `<bpmn:callActivity>` owner, one bounded top-level `errorEventDefinition` end path that terminates the instance in failed state, one bounded transaction cancel end path with exactly one nested `cancelEventDefinition` end event plus the matching parent cancel boundary, one bounded transaction error end path with one or more nested `errorEventDefinition` end events where each end event has every matching parent error boundary on that same transaction owner whose optional `errorRef` either matches the thrown error or stays omitted as a catch-all, one bounded embedded-subprocess error end path with one or more nested `errorEventDefinition` end events where each end event has every matching parent error boundary on that same embedded subprocess owner whose optional `errorRef` either matches the thrown error or stays omitted as a catch-all, one bounded embedded-subprocess interrupting external-boundary path where one parent timer/message/signal/conditional boundary stays armed while the child shell runs and may cancel that child shell before restoring the parent frame, one bounded embedded-subprocess mixed-boundary path where that same owner may expose one interrupting timer/message/signal/conditional boundary plus one or more interrupting error boundaries and runtime may route through either supported interrupting winner while clearing non-selected siblings, one bounded same-package call-activity interrupting external-boundary path where one parent timer/message/signal/conditional boundary stays armed while the called child process runs and may cancel that child process before restoring the parent frame, one bounded same-package call-activity mixed-boundary path where that same owner may expose one interrupting timer/message/signal/conditional boundary plus one or more interrupting error boundaries and runtime may route through either supported interrupting winner while clearing non-selected siblings, one bounded same-package call-activity error path where one or more child-process `errorEventDefinition` end events route through every matching parent error boundary on that same call-activity owner whose optional `errorRef` either matches the thrown error or stays omitted as a catch-all, one bounded transaction-shell interrupting external-boundary path where one parent timer/message/signal/conditional boundary stays armed while the child shell runs and may cancel that child shell before restoring the parent frame, one bounded transaction-shell mixed-owner path where that same owner may expose one interrupting timer/message/signal/conditional boundary plus one interrupting cancel boundary, one or more interrupting error boundaries, or both at once and runtime may route through either supported interrupting winner while clearing non-selected siblings, one bounded compensation binding inside one bounded `<transaction>` shell using one compensation boundary attached to one completed host-blocking activity plus one detached `isForCompensation=\"true\"` host-blocking handler activity reached through one association, one throw-compensation end event inside one bounded `<transaction>` shell that either stays synchronous or sets `waitForCompletion=\"false\"` while using explicit `activityRef` to target one already compensable activity or omits `activityRef` to replay every already compensable activity in reverse completion order, one throw-compensation intermediate event inside one bounded `<transaction>` shell that either stays synchronous or sets `waitForCompletion=\"false\"` for fire-and-continue routing while using explicit `activityRef` to target one already compensable activity or omits `activityRef` to replay every already compensable activity in reverse completion order before normal sequence-flow routing resumes, one bounded embedded `subProcess` body with exactly one nested `startEvent` and at least one nested `endEvent`, one bounded `<transaction>` shell with exactly one nested `startEvent` and at least one nested `endEvent`, one bounded callActivity that targets another executable process in the same BPMN package, serviceTask, scriptTask, userTask, manualTask, businessRuleTask, those same host-blocking task kinds with bounded `standardLoopCharacteristics`, those same host-blocking task kinds with bounded `multiInstanceLoopCharacteristics` in sequential (`isSequential=\"true\"`) or bounded parallel (omitted or `isSequential=\"false\"`) mode using either integer `loopCardinality` or one collection binding with `loopDataInputRef` plus `inputDataItem`, optional paired `loopDataOutputRef` plus `outputDataItem`, and bounded `completionCondition`, exclusiveGateway with simple boolean-path or numeric-comparison `sequenceFlow` `conditionExpression` values plus one optional `default` flow, structured inclusiveGateway with the same bounded condition/default subset plus one linear matching join fragment, parallelGateway, one exclusive eventBasedGateway whose outgoing targets are message/signal/timer/conditional intermediateCatchEvent waits, and sequenceFlow. Do not introduce in-engine script evaluation, correlations, broader collaboration-aware message routing, broader unstructured inclusive joins, non-interrupting boundaries on unsupported repeating task owners or on transaction shells, embedded subprocess shells or call activities, broader mixed interrupting timer/message/signal/conditional transaction-shell families that exceed one interrupting timer/message/signal/conditional boundary, more than one interrupting cancel boundary, or otherwise exceed the bounded external-plus-cancel-plus-error transaction-shell subset, non-interrupting message, signal, timer, or conditional boundaries on embedded subprocess shells or call activities, cancel ends outside one bounded transaction shell, error ends outside one bounded supported error path or top-level terminal-failure path, more than one cancel boundary on the same transaction owner, compensation event subprocesses, broader mixed boundary families on one embedded subprocess owner or one same-package call-activity owner beyond one interrupting timer/message/signal/conditional boundary plus one or more interrupting error boundaries, in-place multi-instance output bindings where `loopDataOutputRef` equals `loopDataInputRef`, or broader FEEL/script-backed gateway conditions in this bounded slice."
        ),
        json!({
            "source_id": source_id,
            "process_id": process_id,
            "element": element,
        }),
    )
}

fn unsupported_complex_gateway_issue(source_id: &str, process_id: &str) -> LintIssue {
    LintIssue::from_parts(
        "bpmn.unsupported_complex_gateway",
        "Complex gateway execution is deferred",
        format!(
            "Process '{process_id}' in source '{source_id}' uses unsupported element '<complexGateway>'."
        ),
        "BPMN complex gateways can combine custom activation, fan-in, and fan-out rules. The bounded engine does not execute those semantics yet, so complex gateways must be remodeled into one supported gateway family before runtime validation.",
        vec![
            "Use `exclusiveGateway` when exactly one conditional branch should win.".to_string(),
            "Use the bounded `inclusiveGateway` subset when one or more conditionally selected branches must rejoin through the supported structured join shape.".to_string(),
            "Use `parallelGateway` when every branch should run and then synchronize deterministically.".to_string(),
            "Use `eventBasedGateway` when the route is an exclusive race over supported message, signal, timer, or conditional waits.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by replacing `<complexGateway>` in process '{process_id}' with a supported bounded gateway structure: `exclusiveGateway` for one winning branch, structured `inclusiveGateway` for one-or-more branches with a supported join, `parallelGateway` for deterministic all-branch fan-out/fan-in, or `eventBasedGateway` for an exclusive race over supported waits. Preserve the branch intent, but do not rely on complex gateway activation conditions or unstructured synchronization."
        ),
        json!({
            "source_id": source_id,
            "process_id": process_id,
            "element": "complexGateway",
            "deferred_semantics": [
                "activation_condition",
                "custom_fan_in",
                "custom_fan_out",
                "unstructured_synchronization"
            ],
            "recommended_rewrites": [
                "exclusiveGateway",
                "inclusiveGateway",
                "parallelGateway",
                "eventBasedGateway"
            ],
        }),
    )
    .with_structured_repair(json!({
        "schema_version": 1,
        "contract": "bpmn.native.gateway.complex_deferred.v1",
        "strategy": "replace_complex_gateway_with_bounded_gateway",
        "actions": [{
            "op": "replace_element",
            "from": "complexGateway",
            "to_options": [
                "exclusiveGateway",
                "inclusiveGateway",
                "parallelGateway",
                "eventBasedGateway"
            ],
            "preserve": "branch intent and BPMN ids where possible",
            "forbid": "complex gateway activation conditions or unstructured synchronization"
        }]
    }))
}

fn missing_process_definitions_issue(source_id: &str) -> LintIssue {
    LintIssue::from_parts(
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
