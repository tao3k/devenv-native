use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint::bpmn::document_surface::summary::document_surface_evidence;
use crate::lint_api::LintIssue;

pub(super) fn diagram_issue(source: &BpmnSourceFile, tag: &str) -> LintIssue {
    let source_id = &source.source_id;
    LintIssue::from_parts(
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
