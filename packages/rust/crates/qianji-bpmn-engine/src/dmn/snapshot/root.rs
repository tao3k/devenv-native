use super::xml::{attribute_value, local_name};
use crate::dmn_model_api::{DmnRootSnapshot, DmnSourceFile};
use crate::error::{BpmnEngineError, Result};
use quick_xml::Reader;
use quick_xml::events::BytesStart;

pub(super) fn empty_root_snapshot() -> DmnRootSnapshot {
    DmnRootSnapshot {
        element_name: String::new(),
        definitions_id: None,
        name: None,
        namespace: None,
        model_namespace_uri: None,
        model_version_hint: None,
        import_count: 0,
        item_definition_count: 0,
        item_definitions: Vec::new(),
        input_data_count: 0,
        input_data: Vec::new(),
        knowledge_source_count: 0,
        knowledge_sources: Vec::new(),
        business_knowledge_model_count: 0,
        business_knowledge_models: Vec::new(),
        decision_service_count: 0,
        decision_services: Vec::new(),
        organization_unit_count: 0,
        organization_units: Vec::new(),
        performance_indicator_count: 0,
        performance_indicators: Vec::new(),
        text_annotation_count: 0,
        text_annotations: Vec::new(),
        association_count: 0,
        associations: Vec::new(),
        element_collection_count: 0,
        element_collections: Vec::new(),
        group_count: 0,
        groups: Vec::new(),
        dmndi_count: 0,
        dmndi_blocks: Vec::new(),
    }
}

pub(super) fn build_root_snapshot(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<DmnRootSnapshot> {
    let model_namespace_uri = find_model_namespace_uri(source, reader, event)?;
    let model_version_hint = model_namespace_uri
        .as_deref()
        .and_then(extract_model_version_hint);
    Ok(DmnRootSnapshot {
        element_name: local_name(event.name().as_ref()).to_string(),
        definitions_id: attribute_value(source, reader, event, "id")?,
        name: attribute_value(source, reader, event, "name")?,
        namespace: attribute_value(source, reader, event, "namespace")?,
        model_namespace_uri,
        model_version_hint,
        import_count: 0,
        item_definition_count: 0,
        item_definitions: Vec::new(),
        input_data_count: 0,
        input_data: Vec::new(),
        knowledge_source_count: 0,
        knowledge_sources: Vec::new(),
        business_knowledge_model_count: 0,
        business_knowledge_models: Vec::new(),
        decision_service_count: 0,
        decision_services: Vec::new(),
        organization_unit_count: 0,
        organization_units: Vec::new(),
        performance_indicator_count: 0,
        performance_indicators: Vec::new(),
        text_annotation_count: 0,
        text_annotations: Vec::new(),
        association_count: 0,
        associations: Vec::new(),
        element_collection_count: 0,
        element_collections: Vec::new(),
        group_count: 0,
        groups: Vec::new(),
        dmndi_count: 0,
        dmndi_blocks: Vec::new(),
    })
}

fn find_model_namespace_uri(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<Option<String>> {
    for attribute in event.attributes() {
        let attribute = attribute.map_err(|error| BpmnEngineError::InvalidDmnXml {
            source_id: source.source_id.clone(),
            message: error.to_string(),
        })?;
        let key = attribute.key.as_ref();
        if key != b"xmlns" && !key.starts_with(b"xmlns:") {
            continue;
        }
        let value = attribute
            .decode_and_unescape_value(reader.decoder())
            .map_err(|error| BpmnEngineError::InvalidDmnXml {
                source_id: source.source_id.clone(),
                message: error.to_string(),
            })?;
        let value = value.as_ref();
        if looks_like_model_namespace_uri(value) {
            return Ok(Some(value.to_string()));
        }
    }
    Ok(None)
}

fn looks_like_model_namespace_uri(value: &str) -> bool {
    value.contains("/spec/DMN/") && value.contains("/MODEL")
}

fn extract_model_version_hint(model_namespace_uri: &str) -> Option<String> {
    let (_, rest) = model_namespace_uri.split_once("/spec/DMN/")?;
    let version = rest.split("/MODEL").next()?.trim_matches('/');
    if version.is_empty() {
        None
    } else {
        Some(version.to_string())
    }
}
