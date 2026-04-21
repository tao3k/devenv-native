use crate::dmn_model_api::DmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use quick_xml::Reader;
use quick_xml::events::BytesStart;

pub(super) fn required_attribute(
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

pub(super) fn attribute_value(
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
            return Ok(Some(value.into_owned()));
        }
    }
    Ok(None)
}

pub(super) fn local_name(name: &[u8]) -> &str {
    std::str::from_utf8(name)
        .ok()
        .map_or("", |raw| raw.rsplit(':').next().unwrap_or(raw))
}
