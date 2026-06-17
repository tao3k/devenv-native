//! Lightweight code language identifiers for Studio provider boundaries.

use std::path::Path;

/// Normalized source language identifier.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct CodeLanguageId(String);

impl CodeLanguageId {
    /// Return the normalized language identifier.
    #[must_use]
    pub(crate) fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for CodeLanguageId {
    fn from(value: &str) -> Self {
        Self(
            normalize_language_id(value)
                .unwrap_or("unknown")
                .to_string(),
        )
    }
}

pub(crate) fn all_code_language_ids() -> [&'static str; 8] {
    [
        "rust",
        "python",
        "typescript",
        "javascript",
        "julia",
        "modelica",
        "go",
        "java",
    ]
}

pub(crate) fn code_language_id_from_path(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(std::ffi::OsStr::to_str) {
        Some(ext) if ext.eq_ignore_ascii_case("rs") => Some("rust"),
        Some(ext) if ext.eq_ignore_ascii_case("py") => Some("python"),
        Some(ext) if ext.eq_ignore_ascii_case("ts") || ext.eq_ignore_ascii_case("tsx") => {
            Some("typescript")
        }
        Some(ext) if ext.eq_ignore_ascii_case("js") || ext.eq_ignore_ascii_case("jsx") => {
            Some("javascript")
        }
        Some(ext) if ext.eq_ignore_ascii_case("jl") => Some("julia"),
        Some(ext) if ext.eq_ignore_ascii_case("mo") => Some("modelica"),
        Some(ext) if ext.eq_ignore_ascii_case("go") => Some("go"),
        Some(ext) if ext.eq_ignore_ascii_case("java") => Some("java"),
        _ => None,
    }
}

fn normalize_language_id(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "rust" | "rs" => Some("rust"),
        "python" | "py" => Some("python"),
        "typescript" | "ts" | "tsx" => Some("typescript"),
        "javascript" | "js" | "jsx" => Some("javascript"),
        "julia" | "jl" => Some("julia"),
        "modelica" | "mo" => Some("modelica"),
        "go" => Some("go"),
        "java" => Some("java"),
        _ => None,
    }
}
