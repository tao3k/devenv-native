//! Lightweight source language identifiers for retired local AST integration.

use std::path::Path;

/// Source language identifier used by audit suggestion heuristics.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CodeLanguageId(String);

impl CodeLanguageId {
    /// Return the normalized language identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
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

/// Resolve a normalized language identifier from a source path.
#[must_use]
pub fn code_language_id_from_path(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(std::ffi::OsStr::to_str) {
        Some(ext) if ext.eq_ignore_ascii_case("rs") => Some("rust"),
        Some(ext) if ext.eq_ignore_ascii_case("py") => Some("python"),
        Some(ext) if ext.eq_ignore_ascii_case("ts") || ext.eq_ignore_ascii_case("tsx") => {
            Some("typescript")
        }
        Some(ext) if ext.eq_ignore_ascii_case("js") || ext.eq_ignore_ascii_case("jsx") => {
            Some("javascript")
        }
        Some(ext) if ext.eq_ignore_ascii_case("go") => Some("go"),
        Some(ext) if ext.eq_ignore_ascii_case("java") => Some("java"),
        Some(ext) if ext.eq_ignore_ascii_case("c") => Some("c"),
        Some(ext) if ext.eq_ignore_ascii_case("cc") || ext.eq_ignore_ascii_case("cpp") => {
            Some("cpp")
        }
        _ => None,
    }
}

fn normalize_language_id(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "rust" | "rs" => Some("rust"),
        "python" | "py" => Some("python"),
        "typescript" | "ts" | "tsx" => Some("typescript"),
        "javascript" | "js" | "jsx" => Some("javascript"),
        "go" => Some("go"),
        "java" => Some("java"),
        "c" => Some("c"),
        "cpp" | "cc" | "c++" => Some("cpp"),
        _ => None,
    }
}
