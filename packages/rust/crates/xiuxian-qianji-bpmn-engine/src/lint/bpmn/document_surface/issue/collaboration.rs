use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint::bpmn::document_surface::summary::document_surface_evidence;
use crate::lint_api::LintIssue;

pub(super) fn collaboration_issue(source: &BpmnSourceFile, tag: &str) -> LintIssue {
    let source_id = &source.source_id;
    LintIssue::from_parts(
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
