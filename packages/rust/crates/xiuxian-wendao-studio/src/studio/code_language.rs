//! Lightweight code language identifiers for Studio provider boundaries.

/// Return the source language identifiers exposed in Studio capability metadata.
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
