use std::path::Path;

pub(in crate::parsers::markdown) fn normalize_slashes(raw: &str) -> String {
    raw.replace('\\', "/")
}

pub(in crate::parsers::markdown) fn trim_md_extension(raw: &str) -> String {
    let lower = raw.to_lowercase();
    for ext in [".markdown", ".mdx", ".org", ".md"] {
        if lower.ends_with(ext) {
            return raw[..raw.len().saturating_sub(ext.len())].to_string();
        }
    }
    raw.to_string()
}

/// Normalize alias/doc-id key for resolver map.
#[must_use]
pub fn normalize_alias(raw: &str) -> String {
    let cleaned = trim_md_extension(&normalize_slashes(raw.trim()));
    cleaned.trim_matches('/').to_lowercase()
}

/// Whether file extension is supported by the markdown link graph parser.
#[must_use]
pub fn is_supported_note(path: &Path) -> bool {
    path.extension()
        .and_then(|v| v.to_str())
        .is_some_and(|ext| {
            let lower = ext.to_lowercase();
            matches!(lower.as_str(), "md" | "markdown" | "mdx" | "org")
        })
}

pub(crate) fn is_org_note(path: &Path) -> bool {
    path.extension()
        .and_then(|v| v.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("org"))
}

pub(in crate::parsers::markdown) fn relative_doc_id(path: &Path, root: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    let rel_str = normalize_slashes(&rel.to_string_lossy());
    let without_ext = trim_md_extension(&rel_str);
    let out = without_ext.trim_matches('/').to_string();
    if out.is_empty() { None } else { Some(out) }
}
