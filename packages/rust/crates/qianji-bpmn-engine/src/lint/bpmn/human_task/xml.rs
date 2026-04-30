use super::{BpmnSourceFile, BytesStart, Cow, LintSourceDiagnostic, LintSourceSpan, Range, Reader};

pub(super) fn source_diagnostic(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    label: impl Into<String>,
    help: impl Into<String>,
) -> LintSourceDiagnostic {
    let span = event_span(reader, event).unwrap_or(0..0);
    LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(span.start, span.end),
        label,
        help,
    )
}

pub(super) fn source_diagnostic_from_span(
    source: &BpmnSourceFile,
    span: Range<usize>,
    label: impl Into<String>,
    help: impl Into<String>,
) -> LintSourceDiagnostic {
    LintSourceDiagnostic::new(
        &source.source_id,
        LintSourceSpan::new(span.start, span.end),
        label,
        help,
    )
}

pub(super) fn event_span(reader: &Reader<&[u8]>, event: &BytesStart<'_>) -> Option<Range<usize>> {
    let event_end = usize::try_from(reader.buffer_position()).ok()?;
    let raw: &[u8] = event.as_ref();
    let start = event_end.checked_sub(raw.len() + 2)?;
    Some(start..event_end)
}

pub(super) fn is_global_task(tag: &str) -> bool {
    matches!(
        tag,
        "globalTask"
            | "globalBusinessRuleTask"
            | "globalManualTask"
            | "globalScriptTask"
            | "globalUserTask"
    )
}

pub(super) fn is_human_interaction_task(tag: &str) -> bool {
    matches!(
        tag,
        "userTask" | "manualTask" | "globalUserTask" | "globalManualTask"
    )
}

pub(super) fn is_assignment_role(tag: &str) -> bool {
    matches!(
        tag,
        "humanPerformer" | "potentialOwner" | "performer" | "resourceRole"
    )
}

pub(super) fn is_unsupported_assignment_role(tag: &str) -> bool {
    matches!(tag, "performer" | "resourceRole")
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

pub(super) fn local_name(raw: &[u8]) -> &str {
    std::str::from_utf8(raw)
        .ok()
        .map_or("", |name| name.rsplit(':').next().unwrap_or(name))
}
