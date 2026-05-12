//! `parsers::zhixing::tasks::identity` owns Wendao zhixing tasks identity behavior.

/// Normalize a stable identity token for zhixing task entities.
#[must_use]
pub fn normalize_identity_token(raw: &str) -> String {
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
