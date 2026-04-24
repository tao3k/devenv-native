use std::collections::BTreeSet;
use std::path::Path;

use crate::WendaoResourceLinkTarget;

pub(crate) fn is_skill_descriptor_path(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| name == "SKILL.md")
}

pub(crate) fn dedup_targets(targets: &[WendaoResourceLinkTarget]) -> Vec<WendaoResourceLinkTarget> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for target in targets {
        let key = (
            target.target_path.trim().to_string(),
            target
                .reference_type
                .as_deref()
                .map(str::trim)
                .map(str::to_ascii_lowercase)
                .filter(|value: &String| !value.is_empty()),
        );
        if seen.insert(key) {
            deduped.push(target.clone());
        }
    }
    deduped.sort_by(|left, right| left.target_path.cmp(&right.target_path));
    deduped
}

pub(crate) fn normalize_token(raw: &str) -> String {
    let normalized = raw
        .trim()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_ascii_lowercase();
    if normalized.is_empty() {
        "unknown".to_string()
    } else {
        normalized
    }
}
