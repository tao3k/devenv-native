use super::{BpmnSourceFile, LintIssue, document_surface_evidence};

pub(super) fn issue_for_tag(source: &BpmnSourceFile, tag: &str) -> Option<LintIssue> {
    match tag {
        "collaboration"
        | "partnerEntity"
        | "partnerRole"
        | "participant"
        | "messageFlow"
        | "conversation"
        | "choreography"
        | "globalChoreographyTask"
        | "choreographyTask"
        | "subChoreography"
        | "callChoreography" => Some(collaboration_issue(source, tag)),
        "dataObject" | "dataObjectReference" | "dataStore" | "dataStoreReference" => {
            Some(data_artifact_issue(source, tag))
        }
        _ => None,
    }
}

pub(super) fn collaboration_issue(source: &BpmnSourceFile, tag: &str) -> LintIssue {
    let source_id = &source.source_id;
    LintIssue::new(
        "bpmn.unsupported_collaboration_surface",
        "Collaboration, choreography, and pool semantics are deferred",
        format!("Source '{source_id}' contains collaboration-level BPMN element '<{tag}>'."),
        "The bounded engine preserves standard collaboration metadata, but executes one process graph at a time and does not dispatch pools or participants, route message flows or conversations, execute choreography, or match correlation keys and retrieval expressions.",
        vec![
            "Move the executable control flow into one supported `<bpmn:process>` before running it with this engine.".to_string(),
            "Preserve pool, participant, message, conversation, choreography, and correlation declarations as standard BPMN metadata.".to_string(),
            "If cross-pool messaging, correlation, or choreography is required, remodel the current slice as explicit host-dispatched tasks or supported wait events until collaboration routing is implemented.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by removing executable dependency on `<{tag}>`. Keep one supported `<bpmn:process>` with explicit sequence flows, and preserve pool/participant intent as non-executable documentation or host-level routing metadata."
        ),
        document_surface_evidence(source, tag, "collaboration"),
    )
}

pub(super) fn data_artifact_issue(source: &BpmnSourceFile, tag: &str) -> LintIssue {
    let source_id = &source.source_id;
    LintIssue::new(
        "bpmn.unsupported_data_surface",
        "BPMN data-object and data-store semantics are deferred",
        format!("Source '{source_id}' contains BPMN data element '<{tag}>'."),
        "The bounded engine keeps workflow data in JSON variables and host payloads; it does not yet execute BPMN data objects or data stores.",
        vec![
            "Represent runtime data through workflow variables, host-work input/output payloads, or DMN decision inputs.".to_string(),
            "Remove `<bpmn:dataObject*>` and `<bpmn:dataStore*>` dependencies from the executable slice.".to_string(),
            "If the data artifact is documentation-only, keep that meaning outside the executable BPMN subset.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by replacing `<{tag}>` execution semantics with explicit JSON variables, host-work payload fields, or DMN inputs. Preserve workflow intent, but remove BPMN data-object or data-store dependencies from this bounded executable slice."
        ),
        document_surface_evidence(source, tag, "data"),
    )
}
