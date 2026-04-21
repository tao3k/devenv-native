use crate::dmn_model_api::DmnSourceFile;
use crate::dmn_parse_api::parser::state::CaptureTarget;
use crate::error::{BpmnEngineError, Result};
use quick_xml::Reader;
use quick_xml::escape::{resolve_predefined_entity, unescape};
use quick_xml::events::{BytesRef, BytesStart};
use std::borrow::Cow;

pub(crate) fn append_capture_text(
    source: &DmnSourceFile,
    capture_target: Option<&CaptureTarget>,
    capture_buffer: &mut String,
    decoded: std::result::Result<Cow<'_, str>, quick_xml::encoding::EncodingError>,
) -> Result<()> {
    if capture_target.is_none() {
        return Ok(());
    }
    let text = decoded.map_err(|error| BpmnEngineError::InvalidDmnXml {
        source_id: source.source_id.clone(),
        message: error.to_string(),
    })?;
    let text = unescape(text.as_ref()).map_err(|error| BpmnEngineError::InvalidDmnXml {
        source_id: source.source_id.clone(),
        message: error.to_string(),
    })?;
    capture_buffer.push_str(text.as_ref());
    Ok(())
}

pub(crate) fn append_capture_reference(
    source: &DmnSourceFile,
    capture_target: Option<&CaptureTarget>,
    capture_buffer: &mut String,
    reference: &BytesRef<'_>,
) -> Result<()> {
    if capture_target.is_none() {
        return Ok(());
    }
    if let Some(ch) =
        reference
            .resolve_char_ref()
            .map_err(|error| BpmnEngineError::InvalidDmnXml {
                source_id: source.source_id.clone(),
                message: error.to_string(),
            })?
    {
        capture_buffer.push(ch);
        return Ok(());
    }

    let reference = reference
        .decode()
        .map_err(|error| BpmnEngineError::InvalidDmnXml {
            source_id: source.source_id.clone(),
            message: error.to_string(),
        })?;
    let entity = resolve_predefined_entity(reference.as_ref()).ok_or_else(|| {
        BpmnEngineError::InvalidDmnXml {
            source_id: source.source_id.clone(),
            message: format!("unrecognized XML entity reference '&{reference};'"),
        }
    })?;
    capture_buffer.push_str(entity);
    Ok(())
}

pub(crate) fn required_attribute(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    element: &str,
    attribute: &str,
) -> Result<String> {
    attribute_value(source, reader, event, attribute)?.ok_or_else(|| {
        BpmnEngineError::MissingDmnAttribute {
            source_id: source.source_id.clone(),
            element: element.to_string(),
            attribute: attribute.to_string(),
        }
    })
}

pub(crate) fn attribute_value(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    attribute_name: &str,
) -> Result<Option<String>> {
    for attribute in event.attributes() {
        let attribute = attribute.map_err(|error| BpmnEngineError::InvalidDmnXml {
            source_id: source.source_id.clone(),
            message: error.to_string(),
        })?;
        if local_name(attribute.key.as_ref()) == attribute_name {
            let value = attribute
                .decode_and_unescape_value(reader.decoder())
                .map_err(|error| BpmnEngineError::InvalidDmnXml {
                    source_id: source.source_id.clone(),
                    message: error.to_string(),
                })?;
            return Ok(Some(match value {
                Cow::Borrowed(value) => value.to_string(),
                Cow::Owned(value) => value,
            }));
        }
    }
    Ok(None)
}

pub(crate) fn local_name(name: &[u8]) -> &str {
    std::str::from_utf8(name)
        .ok()
        .map_or("", |raw| raw.rsplit(':').next().unwrap_or(raw))
}
