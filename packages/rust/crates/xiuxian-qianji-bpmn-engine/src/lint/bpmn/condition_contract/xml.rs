use super::{BytesStart, Cow, Event, Reader, resolve_predefined_entity};

pub(super) fn append_entity_reference(target: &mut String, reference: Option<&str>) {
    let Some(reference) = reference else {
        return;
    };
    if let Some(entity) = resolve_predefined_entity(reference) {
        target.push_str(entity);
    } else {
        target.push('&');
        target.push_str(reference);
        target.push(';');
    }
}

pub(super) fn find_condition_expression_span(
    contents: &str,
    gateway_id: &str,
    condition: &str,
) -> Option<std::ops::Range<usize>> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut sequence_flow_depth = None;
    let mut depth = 0usize;
    let mut in_condition_expression = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                depth += 1;
                if is_element(&event, "sequenceFlow")
                    && attribute_value(&reader, &event, "sourceRef").as_deref() == Some(gateway_id)
                {
                    sequence_flow_depth = Some(depth);
                } else if sequence_flow_depth.is_some() && is_element(&event, "conditionExpression")
                {
                    in_condition_expression = true;
                }
            }
            Ok(Event::Empty(event)) => {
                if is_element(&event, "sequenceFlow")
                    && attribute_value(&reader, &event, "sourceRef").as_deref() == Some(gateway_id)
                {
                    sequence_flow_depth = None;
                }
            }
            Ok(Event::Text(event)) if in_condition_expression => {
                let text = event.decode().ok()?;
                if text.trim() == condition {
                    return event_text_span(reader_position(&reader)?, event.as_ref());
                }
            }
            Ok(Event::CData(event)) if in_condition_expression => {
                let text = event.decode().ok()?;
                if text.trim() == condition {
                    return event_text_span(reader_position(&reader)?, event.as_ref());
                }
            }
            Ok(Event::End(event)) => {
                if local_name(event.name().as_ref()) == "conditionExpression" {
                    in_condition_expression = false;
                }
                if sequence_flow_depth == Some(depth) {
                    sequence_flow_depth = None;
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}

pub(super) fn is_element(event: &BytesStart<'_>, local: &str) -> bool {
    local_name(event.name().as_ref()) == local
}

pub(super) fn local_name(raw: &[u8]) -> &str {
    std::str::from_utf8(raw)
        .ok()
        .map_or("", |name| name.rsplit(':').next().unwrap_or(name))
}

pub(super) fn attribute_value(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    attribute_name: &str,
) -> Option<String> {
    for attribute in event.attributes().flatten() {
        if local_name(attribute.key.as_ref()) != attribute_name {
            continue;
        }
        let value = attribute.decode_and_unescape_value(reader.decoder()).ok()?;
        return Some(match value {
            Cow::Borrowed(value) => value.to_string(),
            Cow::Owned(value) => value,
        });
    }
    None
}

pub(super) fn reader_position(reader: &Reader<&[u8]>) -> Option<usize> {
    usize::try_from(reader.buffer_position()).ok()
}

pub(super) fn event_text_span(event_end: usize, raw_text: &[u8]) -> Option<std::ops::Range<usize>> {
    let end = event_end;
    let start = end.checked_sub(raw_text.len())?;
    Some(start..end)
}
