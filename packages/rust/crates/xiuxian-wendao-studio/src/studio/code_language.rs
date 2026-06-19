//! Lightweight code language identifiers for Studio provider boundaries.

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
