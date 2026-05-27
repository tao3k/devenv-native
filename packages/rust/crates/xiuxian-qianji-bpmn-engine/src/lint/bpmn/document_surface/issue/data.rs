use crate::bpmn_model_api::{BpmnDocumentSnapshot, BpmnIoSpecificationSnapshot};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::bpmn_snapshot_api::snapshot_bpmn_source;
use crate::lint::bpmn::document_surface::summary::document_surface_evidence;
use crate::lint_api::LintIssue;

pub(super) fn io_set_lifecycle_issue(source: &BpmnSourceFile) -> Option<LintIssue> {
    let snapshot = snapshot_bpmn_source(source).ok()?;
    if has_deferred_io_set_lifecycle(&snapshot) {
        Some(data_artifact_issue(source, "ioSetLifecycle"))
    } else {
        None
    }
}

pub(super) fn data_artifact_issue(source: &BpmnSourceFile, tag: &str) -> LintIssue {
    let source_id = &source.source_id;
    let evidence = document_surface_evidence(source, tag, "data");
    LintIssue::from_parts(
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
        evidence,
    )
}

fn has_deferred_io_set_lifecycle(snapshot: &BpmnDocumentSnapshot) -> bool {
    snapshot
        .processes
        .iter()
        .flat_map(|process| process.io_specifications.iter())
        .any(io_spec_has_deferred_lifecycle)
        || snapshot
            .root
            .global_tasks
            .iter()
            .flat_map(|task| task.io_specifications.iter())
            .any(io_spec_has_deferred_lifecycle)
}

fn io_spec_has_deferred_lifecycle(spec: &BpmnIoSpecificationSnapshot) -> bool {
    spec.input_sets.iter().any(|set| {
        !set.optional_input_refs.is_empty()
            || !set.while_executing_input_refs.is_empty()
            || !set.output_set_refs.is_empty()
    }) || spec.output_sets.iter().any(|set| {
        !set.optional_output_refs.is_empty()
            || !set.while_executing_output_refs.is_empty()
            || !set.input_set_refs.is_empty()
    })
}
