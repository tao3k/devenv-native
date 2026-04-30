use super::{BpmnSourceFile, LintIssue, document_surface_evidence};

pub(super) fn issue_for_tag(
    source: &BpmnSourceFile,
    tag: &str,
    parent: Option<&str>,
) -> Option<LintIssue> {
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
        "dataStore" | "dataStoreReference" => Some(data_artifact_issue(source, tag)),
        "ioSpecification" if parent.is_some_and(|parent| parent == "process") => {
            Some(data_artifact_issue(source, tag))
        }
        "BPMNDiagram" | "BPMNPlane" | "BPMNShape" | "BPMNEdge" | "BPMNLabel" | "BPMNLabelStyle" => {
            Some(diagram_issue(source, tag))
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
        "BPMN data-store persistence semantics are deferred",
        format!("Source '{source_id}' contains BPMN data element '<{tag}>'."),
        "The bounded engine can copy through process-level data objects, but it does not execute BPMN data stores or persistent store references.",
        vec![
            "Represent runtime data through workflow variables, host-work input/output payloads, or DMN decision inputs.".to_string(),
            "Use process-level `<bpmn:dataObject>` and `<bpmn:dataObjectReference>` only for bounded in-instance copy-in/copy-out.".to_string(),
            "Remove `<bpmn:dataStore*>` dependencies from the executable slice until a storage policy exists.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by replacing `<{tag}>` persistence semantics with explicit JSON variables, host-work payload fields, or DMN inputs. Preserve workflow intent, but remove BPMN data-store dependencies from this bounded executable slice."
        ),
        document_surface_evidence(source, tag, "data"),
    )
}

pub(super) fn diagram_issue(source: &BpmnSourceFile, tag: &str) -> LintIssue {
    let source_id = &source.source_id;
    LintIssue::new(
        "bpmn.metadata_di_surface",
        "BPMN diagram interchange is metadata-only",
        format!("Source '{source_id}' contains BPMN diagram-interchange element '<{tag}>'."),
        "The bounded engine preserves BPMN DI layout metadata for round-trip compatibility, but runtime execution does not depend on diagram coordinates, shapes, or label styles.",
        vec![
            "Keep BPMN DI blocks when interchange or visual round-tripping matters.".to_string(),
            "Do not rely on BPMN DI shapes, edges, bounds, waypoints, labels, or fonts for executable runtime semantics.".to_string(),
        ],
        format!(
            "Treat `<{tag}>` in BPMN source '{source_id}' as diagram-interchange metadata only. Preserve the layout block for editor compatibility, and keep executable behavior in standard process flow, events, tasks, gateways, and data mappings."
        ),
        document_surface_evidence(source, tag, "diagram"),
    )
}
