use super::{BytesStart, Cow, Range, Reader, resolve_predefined_entity};

pub(super) fn parse_output_names(text: &str) -> Vec<String> {
    text.split([',', '\n', '\t', ' '])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

pub(super) fn reader_position(reader: &Reader<&[u8]>) -> Option<usize> {
    usize::try_from(reader.buffer_position()).ok()
}

pub(super) fn start_event_span(event_end: usize, event: &BytesStart<'_>) -> Option<Range<usize>> {
    let raw: &[u8] = event.as_ref();
    let start = event_end.checked_sub(raw.len() + 2)?;
    Some(start..event_end)
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

pub(super) fn local_name(name: &[u8]) -> String {
    let raw = std::str::from_utf8(name).unwrap_or_default();
    raw.rsplit_once(':')
        .map_or(raw, |(_, local)| local)
        .to_string()
}

pub(super) fn append_entity_reference(target: &mut String, reference: Option<&str>) {
    if let Some(reference) = reference
        && let Some(resolved) = resolve_predefined_entity(reference)
    {
        target.push_str(resolved);
    }
}
