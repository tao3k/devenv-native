use crate::error::BpmnEngineError;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn issue_from_bpmn_identity_error(error: &BpmnEngineError) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::MissingRequiredNodeElement {
            process_id,
            node_id,
            element,
        } => LintIssue::from_parts(
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
        } => LintIssue::from_parts(
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
