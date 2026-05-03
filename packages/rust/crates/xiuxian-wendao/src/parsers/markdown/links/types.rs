#[derive(Debug, Default)]
pub(in crate::parsers::markdown) struct ExtractedLinkTargets {
    pub note_links: Vec<String>,
    pub attachments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(any(test, feature = "search-runtime"))]
/// Resolved note link metadata used by repository search indexing.
pub struct ResolvedNoteReference {
    /// Repository-relative target note path or identifier.
    pub note_target: String,
    /// Optional explicit target address such as a heading or block id.
    pub target_address: Option<String>,
    /// Original markdown reference text.
    pub original: String,
}

#[derive(Debug)]
pub(in crate::parsers::markdown) enum ParsedTarget {
    Note(String),
    Attachment(String),
}
