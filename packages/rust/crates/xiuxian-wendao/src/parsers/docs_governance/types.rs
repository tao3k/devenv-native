//! Shared parser-owned helper types for docs governance parsing.

/// A slice of a line in a document.
#[derive(Debug, Clone, Copy)]
pub struct LineSlice<'a> {
    /// 1-based source line number.
    pub line_number: usize,
    /// Byte offset where this line starts.
    pub start_offset: usize,
    /// Byte offset where this line ends.
    pub end_offset: usize,
    /// Trimmed line contents without surrounding whitespace.
    pub trimmed: &'a str,
    /// Original line contents without trailing newline bytes.
    pub without_newline: &'a str,
    /// Trailing newline sequence captured for this line.
    pub newline: &'a str,
}

/// Parsed top properties drawer.
#[derive(Debug, Clone, Copy)]
pub struct TopPropertiesDrawer<'a> {
    /// 1-based line number where the drawer starts.
    pub properties_line: usize,
    /// Byte offset where a missing `:ID:` line should be inserted.
    pub insert_offset: usize,
    /// Newline sequence used by the surrounding document.
    pub newline: &'a str,
    /// Parsed `:ID:` line when one is already present.
    pub id_line: Option<IdLine<'a>>,
}

/// Parsed `:ID:` line in a properties drawer.
#[derive(Debug, Clone, Copy)]
pub struct IdLine<'a> {
    /// 1-based source line number.
    pub line: usize,
    /// Parsed `:ID:` value.
    pub value: &'a str,
    /// Byte offset where the value starts.
    pub value_start: usize,
    /// Byte offset where the value ends.
    pub value_end: usize,
}

/// Parsed `:LINKS:` line in a relations block.
#[derive(Debug, Clone, Copy)]
pub struct LinksLine<'a> {
    /// 1-based source line number.
    pub line: usize,
    /// Raw `:LINKS:` payload.
    pub value: &'a str,
    /// Byte offset where the payload starts.
    pub value_start: usize,
    /// Byte offset where the payload ends.
    pub value_end: usize,
}

/// Parsed `:FOOTER:` block.
#[derive(Debug, Clone, Copy)]
pub struct FooterBlock<'a> {
    /// 1-based source line number where the footer starts.
    pub line: usize,
    /// Byte offset where the footer block starts.
    pub start_offset: usize,
    /// Byte offset where the footer block ends.
    pub end_offset: usize,
    /// Parsed `:STANDARDS:` value, when present.
    pub standards_value: Option<&'a str>,
    /// Parsed `:LAST_SYNC:` value, when present.
    pub last_sync_value: Option<&'a str>,
}

/// Hidden workspace-path link occurrence extracted from a canonical document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HiddenPathLink {
    /// 1-based source line number.
    pub line: usize,
    /// Byte offset where the link starts.
    pub start_offset: usize,
    /// Byte offset where the link ends.
    pub end_offset: usize,
    /// Original markup for the offending link.
    pub link_markup: String,
    /// Normalized hidden target path.
    pub target: String,
}
