//! Org note aggregate parsing.

use crate::note::NoteCore;

use super::types::OrgNote;

/// Parse a parser-owned Org note aggregate from raw content.
#[must_use]
pub fn parse_org_note(content: &str, fallback_title: &str) -> OrgNote {
    let document = super::document::parse_org_document(content, fallback_title);
    let sections = super::sections::extract_org_sections(document.core.body.as_str());
    OrgNote {
        document,
        core: NoteCore {
            references: Vec::new(),
            targets: Vec::new(),
            sections,
        },
    }
}
