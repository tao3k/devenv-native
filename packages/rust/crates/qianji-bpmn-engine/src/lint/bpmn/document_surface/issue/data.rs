use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint::bpmn::document_surface::data::data_store_binding_count_from_evidence;
use crate::lint::bpmn::document_surface::summary::document_surface_evidence;
use crate::lint_api::LintIssue;

pub(super) fn data_artifact_issue(source: &BpmnSourceFile, tag: &str) -> LintIssue {
    let source_id = &source.source_id;
    let evidence = document_surface_evidence(source, tag, "data");
    if data_store_binding_count_from_evidence(&evidence) > 0 {
        return data_store_binding_issue(source, tag, evidence);
    }
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

fn data_store_binding_issue(
    source: &BpmnSourceFile,
    tag: &str,
    evidence: serde_json::Value,
) -> LintIssue {
    let source_id = &source.source_id;
    LintIssue::from_parts(
        "bpmn.unsupported_data_store_binding",
        "BPMN data-store bindings are not executable",
        format!(
            "Source '{source_id}' contains BPMN data-store element '<{tag}>' used by an executable data association."
        ),
        "The bounded engine preserves BPMN data-store metadata, but a data association that binds through `dataStoreReference` implies persistent read or write behavior. That storage and transaction policy is not executable in this runtime slice.",
        vec![
            "Replace the executable data-store association with workflow variables or bounded `<bpmn:dataObjectReference>` mappings.".to_string(),
            "Keep `<bpmn:dataStore>` and `<bpmn:dataStoreReference>` only as metadata until a storage policy exists.".to_string(),
            "Route persistence through an explicit host-dispatched task payload if external storage access is required.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' by replacing `<{tag}>` data-store bindings with workflow variables, bounded data-object references, or explicit host-work input/output fields. Preserve the standard BPMN data-store metadata only when it is not required for runtime execution."
        ),
        evidence,
    )
}
