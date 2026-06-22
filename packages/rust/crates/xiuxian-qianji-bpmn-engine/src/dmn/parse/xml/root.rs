use super::decode::{local_name, required_attribute};
use crate::dmn_model_api::DmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use quick_xml::Reader;
use quick_xml::events::BytesStart;

const SUPPORTED_DMN_MODEL_VERSIONS: &[&str] = &["20180521", "20191111"];

pub(crate) fn validate_dmn_root_start_tag(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<()> {
    let event_name = event.name();
    let root_element = local_name(event_name.as_ref());
    if root_element != "definitions" {
        return Err(BpmnEngineError::UnsupportedDmnRootElement {
            source_id: (source.source_id.clone()).into(),
            element: root_element.to_string(),
        });
    }

    let _ = required_attribute(source, reader, event, "definitions", "name")?;
    let _ = required_attribute(source, reader, event, "definitions", "namespace")?;

    let Some(model_namespace_uri) = find_model_namespace_uri(source, reader, event)? else {
        return Err(BpmnEngineError::MissingDmnModelNamespace {
            source_id: (source.source_id.clone()).into(),
        });
    };

    if !is_supported_dmn_model_namespace(&model_namespace_uri) {
        return Err(BpmnEngineError::UnsupportedDmnModelNamespace {
            source_id: (source.source_id.clone()).into(),
            model_namespace_uri,
        });
    }

    Ok(())
}

fn find_model_namespace_uri(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<Option<String>> {
    for attribute in event.attributes() {
        let attribute = attribute.map_err(|error| BpmnEngineError::InvalidDmnXml {
            source_id: (source.source_id.clone()).into(),
            message: error.to_string(),
        })?;
        let key = attribute.key.as_ref();
        if key != b"xmlns" && !key.starts_with(b"xmlns:") {
            continue;
        }
        let value = attribute
            .decode_and_unescape_value(reader.decoder())
            .map_err(|error| BpmnEngineError::InvalidDmnXml {
                source_id: (source.source_id.clone()).into(),
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

fn extract_model_version_hint(model_namespace_uri: &str) -> Option<&str> {
    let (_, rest) = model_namespace_uri.split_once("/spec/DMN/")?;
    let version = rest.split("/MODEL").next()?.trim_matches('/');
    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}

fn is_supported_dmn_model_namespace(model_namespace_uri: &str) -> bool {
    extract_model_version_hint(model_namespace_uri)
        .is_some_and(|version| SUPPORTED_DMN_MODEL_VERSIONS.contains(&version))
}
