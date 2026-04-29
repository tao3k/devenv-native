use super::{
    BpmnProcessSpec, BytesStart, Cow, ProcessMetadata, Range, Reader, resolve_predefined_entity,
};

pub(super) fn outgoing_edge_indices(
    process: &BpmnProcessSpec,
    node_index: usize,
) -> Option<&[u32]> {
    let node_index = u32::try_from(node_index).ok()?;
    Some(process.outgoing_edge_indices(node_index))
}

pub(super) fn is_task_tag(tag: &str) -> bool {
    matches!(
        tag,
        "serviceTask"
            | "userTask"
            | "manualTask"
            | "businessRuleTask"
            | "scriptTask"
            | "sendTask"
            | "receiveTask"
    )
}

pub(super) fn is_span_only_node_tag(tag: &str) -> bool {
    matches!(
        tag,
        "exclusiveGateway"
            | "inclusiveGateway"
            | "parallelGateway"
            | "eventBasedGateway"
            | "startEvent"
            | "endEvent"
    )
}

pub(super) fn record_node_span(
    metadata: &mut ProcessMetadata,
    contents: &str,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    node_id: &str,
) {
    if let Some(event_end) = reader_position(reader)
        && let Some(span) = start_or_empty_event_span(contents, event_end, event)
    {
        metadata.node_spans.insert(node_id.to_string(), span);
    }
}

pub(super) fn start_or_empty_event_span(
    contents: &str,
    event_end: usize,
    event: &BytesStart<'_>,
) -> Option<Range<usize>> {
    let raw: &[u8] = event.as_ref();
    [2, 3].into_iter().find_map(|extra| {
        let start = event_end.checked_sub(raw.len() + extra)?;
        contents
            .as_bytes()
            .get(start)
            .is_some_and(|byte| *byte == b'<')
            .then_some(start..event_end)
    })
}

pub(super) fn parse_variable_names(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

pub(super) fn reader_position(reader: &Reader<&[u8]>) -> Option<usize> {
    usize::try_from(reader.buffer_position()).ok()
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
