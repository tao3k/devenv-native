use crate::bpmn_parse_api::BpmnSourceFile;
use crate::dmn_model_api::DmnDecisionRef;
use crate::error::{BpmnEngineError, Result};
use quick_xml::Reader;
use quick_xml::events::BytesStart;
use std::borrow::Cow;

pub(super) fn event_reference_id(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
) -> Result<Option<String>> {
    let attribute_name = match tag {
        "messageEventDefinition" => "messageRef",
        "signalEventDefinition" => "signalRef",
        "errorEventDefinition" => "errorRef",
        "compensateEventDefinition" => "activityRef",
        _ => return Ok(None),
    };
    attribute_value(reader, event, attribute_name)
}

pub(super) fn required_attribute(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    element: &str,
    attribute: &str,
) -> Result<String> {
    attribute_value(reader, event, attribute)?.ok_or_else(|| BpmnEngineError::MissingAttribute {
        source_id: source.source_id.clone(),
        element: element.to_string(),
        attribute: attribute.to_string(),
    })
}

pub(super) fn attribute_value(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    attribute_name: &str,
) -> Result<Option<String>> {
    for attribute in event.attributes() {
        let attribute =
            attribute.map_err(|error| BpmnEngineError::CheckpointCodec(error.to_string()))?;
        if local_name(attribute.key.as_ref()) == attribute_name {
            let value = attribute
                .decode_and_unescape_value(reader.decoder())
                .map_err(|error| BpmnEngineError::CheckpointCodec(error.to_string()))?;
            return Ok(Some(match value {
                Cow::Borrowed(value) => value.to_string(),
                Cow::Owned(value) => value,
            }));
        }
    }
    Ok(None)
}

pub(super) fn boolean_attribute_value(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    attribute_name: &str,
) -> Result<Option<bool>> {
    Ok(
        match attribute_value(reader, event, attribute_name)?.as_deref() {
            None => None,
            Some("true" | "1") => Some(true),
            Some(_) => Some(false),
        },
    )
}

pub(super) fn parse_optional_u32_attribute(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    attribute_name: &str,
    process_id: &str,
    node_id: &str,
    detail: &'static str,
) -> Result<Option<u32>> {
    attribute_value(reader, event, attribute_name)?
        .map(|value| {
            value
                .parse::<u32>()
                .map_err(|_| BpmnEngineError::UnsupportedLoopConfiguration {
                    process_id: process_id.to_string(),
                    node_id: node_id.to_string(),
                    detail,
                })
        })
        .transpose()
}

pub(super) fn decision_reference(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<Option<DmnDecisionRef>> {
    let Some(decision_id) = attribute_value(reader, event, "decisionRef")? else {
        return Ok(None);
    };
    let decision = match attribute_value(reader, event, "decisionRefSource")? {
        Some(source_id) => DmnDecisionRef::new(decision_id).with_source_id(source_id),
        None => DmnDecisionRef::new(decision_id),
    };
    Ok(Some(decision))
}

pub(super) fn cancel_activity_value(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<bool> {
    Ok(!matches!(
        attribute_value(reader, event, "cancelActivity")?.as_deref(),
        Some("false" | "0")
    ))
}

pub(super) fn local_name(name: &[u8]) -> &str {
    std::str::from_utf8(name)
        .ok()
        .map_or("", |raw| raw.rsplit(':').next().unwrap_or(raw))
}
