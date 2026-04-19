#[derive(Debug, Default)]
pub(in crate::parsers::markdown) struct ExtractedLinkTargets {
    pub note_links: Vec<String>,
    pub attachments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(any(test, feature = "studio"))]
pub(crate) struct ResolvedNoteReference {
    pub note_target: String,
    pub target_address: Option<String>,
    pub original: String,
}

#[derive(Debug)]
pub(in crate::parsers::markdown) enum ParsedTarget {
    Note(String),
    Attachment(String),
}
