use crate::document::OrgDocument;
use crate::note::{NoteAggregate, NoteCore};
use crate::references::MarkdownReference;
use crate::sections::SectionCore;
use crate::targets::MarkdownTargetOccurrence;
use crate::toc::TocDocument;

/// Org-local name for the shared parser-owned section core.
pub type OrgSection = SectionCore;

/// Org-specific note-body aggregation over parser-owned item contracts.
pub type OrgNoteCore = NoteCore<MarkdownReference, MarkdownTargetOccurrence, OrgSection>;

/// Parser-owned aggregate for one Org note body.
pub type OrgNote =
    NoteAggregate<OrgDocument, MarkdownReference, MarkdownTargetOccurrence, OrgSection>;

/// Parser-owned aggregate for one Org TOC/body structure.
pub type OrgTocDocument = TocDocument<OrgDocument, OrgSection>;

/// Parse one parser-owned Org TOC surface from raw content.
#[must_use]
pub fn parse_org_toc(content: &str, fallback_title: &str) -> OrgTocDocument {
    let document = super::document::parse_org_document(content, fallback_title);
    let sections = super::sections::extract_org_sections(document.core.body.as_str());
    TocDocument { document, sections }
}
