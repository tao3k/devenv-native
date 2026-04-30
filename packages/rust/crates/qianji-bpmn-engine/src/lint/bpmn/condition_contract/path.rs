use super::{AmbiguousBooleanPathKind, Event, HashSet, Reader, attribute_value, is_element};

pub(super) fn collect_gateway_ids(contents: &str) -> HashSet<String> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(false);
    let mut ids = HashSet::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event) | Event::Empty(event))
                if is_element(&event, "exclusiveGateway")
                    || is_element(&event, "inclusiveGateway") =>
            {
                if let Some(id) = attribute_value(&reader, &event, "id") {
                    ids.insert(id);
                }
            }
            Ok(Event::Eof) | Err(_) => return ids,
            Ok(_) => {}
        }
    }
}

pub(super) fn is_boolean_interaction_choice_value(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true"
            | "false"
            | "yes"
            | "no"
            | "y"
            | "n"
            | "approved"
            | "approve"
            | "rejected"
            | "reject"
            | "accepted"
            | "accept"
            | "confirmed"
            | "confirm"
            | "continue"
            | "proceed"
            | "revise"
            | "revision"
            | "changes"
            | "declined"
            | "decline"
            | "denied"
            | "deny"
            | "stop"
            | "cancel"
            | "cancelled"
    )
}

pub(super) fn is_count_like_boolean_path(path: &str) -> bool {
    let segment = path.rsplit('.').next().unwrap_or(path);
    let normalized = segment.to_ascii_lowercase();
    !is_boolean_shaped_name(&normalized)
        && [
            "count",
            "number",
            "total",
            "index",
            "length",
            "size",
            "amount",
            "remaining",
        ]
        .iter()
        .any(|marker| normalized.contains(marker))
}

pub(super) fn is_content_like_boolean_path(path: &str) -> bool {
    let segment = path.rsplit('.').next().unwrap_or(path);
    let normalized = segment.to_ascii_lowercase();
    !is_boolean_shaped_name(&normalized)
        && !has_embedded_boolean_marker(segment, &normalized)
        && [
            "answer",
            "answers",
            "choice",
            "choices",
            "concern",
            "concerns",
            "detail",
            "details",
            "feedback",
            "guidance",
            "issue",
            "issues",
            "question",
            "questions",
            "response",
            "responses",
            "result",
            "results",
            "status",
        ]
        .iter()
        .any(|marker| normalized.ends_with(marker))
}

pub(super) fn has_embedded_boolean_marker(segment: &str, normalized: &str) -> bool {
    ["Is", "Has", "Can", "Should", "Needs", "Need", "Did", "Will"]
        .iter()
        .any(|marker| segment.contains(marker))
        || [
            "_is_", "_has_", "_can_", "_should_", "_needs_", "_need_", "_did_", "_will_", "-is-",
            "-has-", "-can-", "-should-", "-needs-", "-need-", "-did-", "-will-",
        ]
        .iter()
        .any(|marker| normalized.contains(marker))
}

pub(super) fn ambiguous_boolean_path_kind(path: &str) -> Option<AmbiguousBooleanPathKind> {
    if is_count_like_boolean_path(path) {
        return Some(AmbiguousBooleanPathKind::CountLike);
    }
    if is_content_like_boolean_path(path) {
        return Some(AmbiguousBooleanPathKind::ContentLike);
    }
    None
}

pub(super) fn is_boolean_shaped_name(normalized: &str) -> bool {
    ["is", "has", "can", "should", "needs", "need", "did", "will"]
        .iter()
        .any(|prefix| normalized.starts_with(prefix))
}
