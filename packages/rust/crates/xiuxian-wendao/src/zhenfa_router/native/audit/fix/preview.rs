//! `zhenfa_router::native::audit::fix::preview` owns Wendao audit fix preview behavior.

/// Preview of a single fix operation.
#[derive(Debug, Clone)]
pub struct FixPreview {
    /// Line number where the fix applies.
    pub line_number: usize,
    /// Original content to be replaced.
    pub original: String,
    /// Replacement content.
    pub replacement: String,
    /// Confidence score for this fix.
    pub confidence: f32,
    /// Whether this is a surgical (byte-precise) fix.
    pub is_surgical: bool,
    /// Full preview of the file content after applying this fix.
    pub preview_content: String,
}

impl std::fmt::Display for FixPreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Line {}: (confidence: {:.0}%)",
            self.line_number,
            self.confidence * 100.0
        )?;
        writeln!(f, "  - {}", self.original)?;
        writeln!(f, "  + {}", self.replacement)?;
        if self.is_surgical {
            write!(f, "  [surgical: byte-precise]")
        } else {
            write!(f, "  [legacy: string search]")
        }
    }
}
