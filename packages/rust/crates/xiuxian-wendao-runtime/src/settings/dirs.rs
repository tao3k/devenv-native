//! Project settings directory resolution for runtime configuration loading.

/// Normalize one relative directory entry for config-driven path lists.
#[must_use]
pub fn normalize_relative_dir(value: &str) -> Option<String> {
    let normalized = value
        .trim()
        .replace('\\', "/")
        .trim_matches('/')
        .to_string();
    if normalized.is_empty() || normalized == "." {
        None
    } else {
        Some(normalized)
    }
}

/// Deduplicate directory entries while preserving original order.
#[must_use]
pub fn dedup_dirs(entries: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for entry in entries {
        let lowered = entry.to_lowercase();
        if seen.insert(lowered) {
            out.push(entry);
        }
    }
    out
}

#[cfg(test)]
#[path = "../../tests/unit/settings/dirs.rs"]
mod tests;
