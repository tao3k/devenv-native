pub(super) fn strip_target_decorations(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(
        trimmed
            .trim_matches(|c: char| c == '[' || c == ']' || c == '(' || c == ')')
            .to_string(),
    )
}

pub(super) fn has_external_scheme(lower: &str) -> bool {
    lower.starts_with("http:") || lower.starts_with("https:") || lower.contains("://")
}

pub(super) fn strip_fragment_and_query(raw: &str) -> &str {
    let without_fragment = raw.split_once('#').map_or(raw, |(base, _)| base);
    without_fragment
        .split_once('?')
        .map_or(without_fragment, |(base, _)| base)
}

pub(super) fn has_supported_note_extension(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "md" | "markdown"))
}

fn resolve_relative_target(
    target: &str,
    source_path: &std::path::Path,
    root: &std::path::Path,
) -> Option<String> {
    let normalized_target = crate::parsers::markdown::paths::normalize_slashes(target.trim());
    if normalized_target.is_empty() {
        return None;
    }

    let resolved_path = if std::path::Path::new(&normalized_target).is_absolute() {
        root.join(normalized_target.trim_start_matches('/'))
    } else {
        source_path
            .parent()
            .unwrap_or(root)
            .join(normalized_target.as_str())
    };

    let relative_path = resolved_path
        .strip_prefix(root)
        .unwrap_or(resolved_path.as_path());
    let relative =
        crate::parsers::markdown::paths::normalize_slashes(&relative_path.to_string_lossy());
    (!relative.is_empty()).then_some(relative)
}

pub(super) fn normalize_markdown_note_target(
    target: &str,
    source_path: &std::path::Path,
    root: &std::path::Path,
) -> Option<String> {
    let resolved = resolve_relative_target(target, source_path, root)?;
    let stem = resolved
        .split_once('.')
        .map_or(resolved.as_str(), |(s, _)| s);
    (!stem.is_empty()).then(|| crate::parsers::markdown::normalize_alias(stem))
}

pub(super) fn normalize_attachment_target(
    target: &str,
    source_path: &std::path::Path,
    root: &std::path::Path,
) -> Option<String> {
    resolve_relative_target(target, source_path, root)
}

pub(super) fn normalize_wikilink_note_target(raw: &str) -> Option<String> {
    let stem = raw.split_once('|').map_or(raw, |(s, _)| s);
    let stem = stem.split_once('#').map_or(stem, |(s, _)| s);
    (!stem.is_empty()).then(|| crate::parsers::markdown::normalize_alias(stem))
}

#[cfg(test)]
#[path = "../../../../tests/unit/parsers/markdown/links/normalize.rs"]
mod tests;
