use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use quick_xml::Reader;
use quick_xml::escape::{resolve_predefined_entity, unescape};
use quick_xml::events::{BytesRef, BytesStart};
use std::borrow::Cow;

pub(in crate::bpmn_snapshot) fn attribute_value(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    attribute_name: &str,
) -> Result<Option<String>> {
    for attribute in event.attributes() {
        let attribute = attribute.map_err(|error| BpmnEngineError::InvalidXml {
            source_id: (source.source_id.clone()).into(),
            message: error.to_string(),
            offset: None,
        })?;
        if local_name(attribute.key.as_ref()) == attribute_name {
            let value = attribute
                .decode_and_unescape_value(reader.decoder())
                .map_err(|error| BpmnEngineError::InvalidXml {
                    source_id: (source.source_id.clone()).into(),
                    message: error.to_string(),
                    offset: None,
                })?;
            return Ok(Some(value.into_owned()));
        }
    }
    Ok(None)
}

pub(in crate::bpmn_snapshot) fn boolean_attribute_value(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    attribute_name: &str,
) -> Result<Option<bool>> {
    Ok(
        match attribute_value(source, reader, event, attribute_name)?.as_deref() {
            None => None,
            Some("true" | "1") => Some(true),
            Some(_) => Some(false),
        },
    )
}

pub(in crate::bpmn_snapshot) fn bpmn_model_namespace(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<Option<String>> {
    for attribute in event.attributes() {
        let attribute = attribute.map_err(|error| BpmnEngineError::InvalidXml {
            source_id: (source.source_id.clone()).into(),
            message: error.to_string(),
            offset: None,
        })?;
        let key = std::str::from_utf8(attribute.key.as_ref()).unwrap_or_default();
        if key != "xmlns" && !key.starts_with("xmlns:") {
            continue;
        }
        let value = attribute
            .decode_and_unescape_value(reader.decoder())
            .map_err(|error| BpmnEngineError::InvalidXml {
                source_id: (source.source_id.clone()).into(),
                message: error.to_string(),
                offset: None,
            })?;
        if value.contains("/BPMN/20100524/MODEL") {
            return Ok(Some(value.into_owned()));
        }
    }
    Ok(None)
}

pub(super) fn append_text_content(
    source: &BpmnSourceFile,
    buffer: &mut String,
    decoded: std::result::Result<Cow<'_, str>, quick_xml::encoding::EncodingError>,
) -> Result<()> {
    let text = decoded.map_err(|error| BpmnEngineError::InvalidXml {
        source_id: (source.source_id.clone()).into(),
        message: error.to_string(),
        offset: None,
    })?;
    let text = unescape(text.as_ref()).map_err(|error| BpmnEngineError::InvalidXml {
        source_id: (source.source_id.clone()).into(),
        message: error.to_string(),
        offset: None,
    })?;
    buffer.push_str(text.as_ref());
    Ok(())
}

pub(super) fn append_reference_content(
    source: &BpmnSourceFile,
    buffer: &mut String,
    reference: &BytesRef<'_>,
) -> Result<()> {
    if let Some(ch) = reference
        .resolve_char_ref()
        .map_err(|error| BpmnEngineError::InvalidXml {
            source_id: (source.source_id.clone()).into(),
            message: error.to_string(),
            offset: None,
        })?
    {
        buffer.push(ch);
        return Ok(());
    }

    let reference = reference
        .decode()
        .map_err(|error| BpmnEngineError::InvalidXml {
            source_id: (source.source_id.clone()).into(),
            message: error.to_string(),
            offset: None,
        })?;
    let entity = resolve_predefined_entity(reference.as_ref()).ok_or_else(|| {
        BpmnEngineError::InvalidXml {
            source_id: (source.source_id.clone()).into(),
            message: format!("unrecognized XML entity reference '&{reference};'"),
            offset: None,
        }
    })?;
    buffer.push_str(entity);
    Ok(())
}

pub(in crate::bpmn_snapshot) fn local_name(name: &[u8]) -> &str {
    std::str::from_utf8(name)
        .ok()
        .map_or("", |raw| raw.rsplit(':').next().unwrap_or(raw))
}
