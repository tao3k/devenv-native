use crate::dmn_model_api::DmnDocumentSnapshot;
use serde_json::{Value, json};

pub(super) fn augment_evidence(
    mut evidence: Value,
    snapshot: Option<&DmnDocumentSnapshot>,
    decision_id: Option<&str>,
) -> Value {
    let Some(snapshot) = snapshot else {
        return evidence;
    };
    let Value::Object(ref mut map) = evidence else {
        return evidence;
    };
    map.insert("document_root".to_string(), json!(snapshot.root));
    map.insert(
        "document_decision_count".to_string(),
        json!(snapshot.decisions.len()),
    );
    if let Some(decision_id) = decision_id
        && let Some(decision) = snapshot.decision(decision_id)
    {
        map.insert("decision_snapshot".to_string(), json!(decision));
    }
    evidence
}

pub(super) fn root_context(snapshot: Option<&DmnDocumentSnapshot>) -> String {
    let Some(snapshot) = snapshot else {
        return String::new();
    };
    let root = &snapshot.root;
    let mut parts = vec![format!(
        "The scanned document root was `<{}>`",
        root.element_name
    )];
    if let Some(model_namespace_uri) = root.model_namespace_uri.as_deref() {
        parts.push(format!("model namespace '{model_namespace_uri}'"));
    }
    if let Some(model_version_hint) = root.model_version_hint.as_deref() {
        parts.push(format!("version hint '{model_version_hint}'"));
    }
    if root.import_count > 0 {
        parts.push(format!("top-level import count {}", root.import_count));
    }
    if root.item_definition_count > 0 {
        parts.push(format!(
            "top-level itemDefinition count {}",
            root.item_definition_count
        ));
    }
    if root.input_data_count > 0 {
        parts.push(format!(
            "top-level inputData count {}",
            root.input_data_count
        ));
    }
    if root.knowledge_source_count > 0 {
        parts.push(format!(
            "top-level knowledgeSource count {}",
            root.knowledge_source_count
        ));
    }
    if root.business_knowledge_model_count > 0 {
        parts.push(format!(
            "top-level businessKnowledgeModel count {}",
            root.business_knowledge_model_count
        ));
    }
    if root.decision_service_count > 0 {
        parts.push(format!(
            "top-level decisionService count {}",
            root.decision_service_count
        ));
    }
    if root.organization_unit_count > 0 {
        parts.push(format!(
            "top-level organizationUnit count {}",
            root.organization_unit_count
        ));
    }
    if root.performance_indicator_count > 0 {
        parts.push(format!(
            "top-level performanceIndicator count {}",
            root.performance_indicator_count
        ));
    }
    if root.text_annotation_count > 0 {
        parts.push(format!(
            "top-level textAnnotation count {}",
            root.text_annotation_count
        ));
    }
    if root.association_count > 0 {
        parts.push(format!(
            "top-level association count {}",
            root.association_count
        ));
    }
    if root.element_collection_count > 0 {
        parts.push(format!(
            "top-level elementCollection count {}",
            root.element_collection_count
        ));
    }
    if root.group_count > 0 {
        parts.push(format!("top-level group count {}", root.group_count));
    }
    if root.dmndi_count > 0 {
        parts.push(format!("top-level DMNDI count {}", root.dmndi_count));
    }
    format!(" {}", parts.join(", ") + ".")
}

pub(super) fn decision_display(
    decision_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> String {
    if let Some(decision_name) = snapshot
        .and_then(|snapshot| snapshot.decision(decision_id))
        .and_then(|decision| decision.name.as_deref())
    {
        format!("Decision '{decision_id}' ('{decision_name}')")
    } else {
        format!("Decision '{decision_id}'")
    }
}
