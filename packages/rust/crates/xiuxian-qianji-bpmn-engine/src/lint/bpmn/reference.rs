use crate::error::BpmnEngineError;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn issue_from_bpmn_reference_error(error: &BpmnEngineError) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::DuplicateProcessId {
            package_id,
            process_id,
        } => LintIssue::from_parts(
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
        BpmnEngineError::DuplicateNodeId { process_id, node_id } => LintIssue::from_parts(
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
        BpmnEngineError::DuplicateSequenceFlowId { process_id, flow_id } => LintIssue::from_parts(
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
        } => LintIssue::from_parts(
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
