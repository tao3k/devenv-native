use std::path::Path;

use xiuxian_wendao_parsers::{
    parse_markdown_note,
    targets::{MarkdownTargetOccurrence, MarkdownTargetOccurrenceKind},
};

use super::{extract_link_targets_from_occurrences, extract_resolved_note_references};

#[test]
fn extract_link_targets_from_occurrences_normalizes_markdown_and_wikilink_targets() {
    let occurrences = vec![
        MarkdownTargetOccurrence {
            kind: MarkdownTargetOccurrenceKind::MarkdownLink,
            target: "docs/guide.md#intro".to_string(),
            byte_range: (0, 20),
            line_range: (1, 1),
        },
        MarkdownTargetOccurrence {
            kind: MarkdownTargetOccurrenceKind::MarkdownImage,
            target: "assets/logo.png".to_string(),
            byte_range: (21, 36),
            line_range: (2, 2),
        },
        MarkdownTargetOccurrence {
            kind: MarkdownTargetOccurrenceKind::WikiLink,
            target: "graph-c".to_string(),
            byte_range: (37, 44),
            line_range: (3, 3),
        },
        MarkdownTargetOccurrence {
            kind: MarkdownTargetOccurrenceKind::MarkdownLink,
            target: "#local".to_string(),
            byte_range: (45, 51),
            line_range: (4, 4),
        },
    ];

    let root = Path::new("/tmp/parser-doc");
    let path = Path::new("/tmp/parser-doc/adapter.md");
    let extracted = extract_link_targets_from_occurrences(&occurrences, path, root);

    assert_eq!(extracted.note_links, vec!["docs/guide", "graph-c"]);
    assert_eq!(extracted.attachments, vec!["assets/logo.png"]);
}

#[test]
fn extract_resolved_note_references_preserves_scoped_addresses() {
    let content = [
        "[Guide Proof](docs/guide.md#^proof-anchor)",
        "[[docs/guide#Overview|Guide Overview]]",
        "[Local Heading](#overview)",
    ]
    .join("\n");
    let note = parse_markdown_note(&content, "Index");
    let root = Path::new("/tmp/parser-doc");
    let source_path = Path::new("/tmp/parser-doc/index.md");

    let resolved = extract_resolved_note_references(
        note.core.references.as_slice(),
        note.core.targets.as_slice(),
        source_path,
        root,
    );

    assert_eq!(
        resolved,
        vec![
            crate::parsers::markdown::ResolvedNoteReference {
                note_target: "docs/guide".to_string(),
                target_address: Some("#^proof-anchor".to_string()),
                original: "[Guide Proof](docs/guide.md#^proof-anchor)".to_string(),
            },
            crate::parsers::markdown::ResolvedNoteReference {
                note_target: "docs/guide".to_string(),
                target_address: Some("#Overview".to_string()),
                original: "[[docs/guide#Overview|Guide Overview]]".to_string(),
            },
        ]
    );
}

#[test]
fn extract_resolved_note_references_ignores_empty_target_occurrences() {
    let content = [
        "[Broken]()",
        "[Guide Proof](docs/guide.md#^proof-anchor)",
        "[[docs/guide#Overview|Guide Overview]]",
    ]
    .join("\n");
    let note = parse_markdown_note(&content, "Index");
    let root = Path::new("/tmp/parser-doc");
    let source_path = Path::new("/tmp/parser-doc/index.md");

    assert_eq!(note.core.targets.len(), 3);
    assert_eq!(note.core.references.len(), 2);

    let resolved = extract_resolved_note_references(
        note.core.references.as_slice(),
        note.core.targets.as_slice(),
        source_path,
        root,
    );

    assert_eq!(
        resolved,
        vec![
            crate::parsers::markdown::ResolvedNoteReference {
                note_target: "docs/guide".to_string(),
                target_address: Some("#^proof-anchor".to_string()),
                original: "[Guide Proof](docs/guide.md#^proof-anchor)".to_string(),
            },
            crate::parsers::markdown::ResolvedNoteReference {
                note_target: "docs/guide".to_string(),
                target_address: Some("#Overview".to_string()),
                original: "[[docs/guide#Overview|Guide Overview]]".to_string(),
            },
        ]
    );
}
