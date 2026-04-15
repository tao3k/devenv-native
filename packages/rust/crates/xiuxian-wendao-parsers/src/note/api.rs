use super::fingerprint::fingerprint_markdown_symbol_surface_with_structure;
use super::types::{MarkdownNote, MarkdownNoteCore, MarkdownNoteParseArtifacts};
use crate::document::parse_markdown_document_from_parts;
use crate::frontmatter::split_frontmatter;
use crate::markdown_structure::{MarkdownStructure, parse_markdown_structure};
use crate::sections::extract_sections_with_structure;

fn build_markdown_note(content: &str, fallback_title: &str) -> (MarkdownNote, MarkdownStructure) {
    let (frontmatter, body) = split_frontmatter(content);
    let structure = parse_markdown_structure(body);
    let document = parse_markdown_document_from_parts(
        frontmatter,
        body,
        fallback_title,
        structure.first_heading_title(),
        structure.lead_snippet(),
        false,
    );
    let sections = extract_sections_with_structure(body, &structure);
    let note = MarkdownNote {
        document,
        core: MarkdownNoteCore {
            references: structure.references().to_vec(),
            targets: structure.targets().to_vec(),
            sections,
        },
    };

    (note, structure)
}

/// Parse a parser-owned Markdown note aggregate from raw content.
#[must_use]
pub fn parse_markdown_note(content: &str, fallback_title: &str) -> MarkdownNote {
    build_markdown_note(content, fallback_title).0
}

/// Parse one parser-owned Markdown note plus the symbol-surface fingerprint
/// derived from the same structural traversal.
#[must_use]
pub fn parse_markdown_note_artifacts(
    content: &str,
    fallback_title: &str,
) -> MarkdownNoteParseArtifacts {
    let (note, structure) = build_markdown_note(content, fallback_title);
    let symbol_fingerprint = fingerprint_markdown_symbol_surface_with_structure(&note, &structure);

    MarkdownNoteParseArtifacts {
        note,
        symbol_fingerprint,
    }
}
