use crate::parsers::markdown::links::extract_link_targets_from_occurrences;
use crate::parsers::markdown::parse_note;
use serde_yaml::Value;
use std::path::Path;
use xiuxian_wendao_parsers::document::{DocumentEnvelope, DocumentFormat};
use xiuxian_wendao_parsers::targets::{
    MarkdownTargetOccurrence, MarkdownTargetOccurrenceKind, TargetOccurrenceCore,
};

#[test]
fn parse_note_uses_parser_owned_document_metadata_contract() {
    let content = concat!(
        "---\n",
        "title: Adapter Contract\n",
        "type: architecture\n",
        "tags:\n",
        "  - parser\n",
        "  - substrate\n",
        "---\n",
        "\n",
        "Lead line for parser-owned metadata.\n",
        "\n",
        "Second line.\n",
    );
    let root = Path::new("/tmp/parser-doc");
    let path = Path::new("/tmp/parser-doc/adapter.md");

    let parsed =
        parse_note(path, root, content).unwrap_or_else(|| panic!("expected parsed note output"));

    assert_eq!(parsed.doc.id, "adapter");
    assert_eq!(parsed.doc.title, "Adapter Contract");
    assert_eq!(parsed.doc.tags, vec!["parser", "substrate"]);
    assert_eq!(parsed.doc.doc_type.as_deref(), Some("architecture"));
    assert_eq!(parsed.doc.lead, "Lead line for parser-owned metadata.");
    assert_eq!(parsed.doc.word_count, 7);
    assert_eq!(
        parsed.doc.search_text,
        "Lead line for parser-owned metadata.\n\nSecond line.\n"
    );
}

#[test]
fn parse_markdown_document_exposes_cross_format_markdown_core() {
    let document = xiuxian_wendao_parsers::document::parse_markdown_document(
        "# Heading Contract\n\nBody text.\n",
        "fallback",
    );
    let envelope: &DocumentEnvelope<Value> = &document;

    assert_eq!(document.core.format, DocumentFormat::Markdown);
    assert_eq!(document.core.title, "Heading Contract");
    assert_eq!(document.core.body, "# Heading Contract\n\nBody text.\n");
    assert!(envelope.raw_metadata.is_none());
}

#[test]
fn parse_note_falls_back_to_heading_title_when_frontmatter_is_missing() {
    let content = "# Heading Contract\n\nBody text.\n";
    let root = Path::new("/tmp/parser-doc");
    let path = Path::new("/tmp/parser-doc/heading.md");

    let parsed =
        parse_note(path, root, content).unwrap_or_else(|| panic!("expected parsed note output"));

    assert_eq!(parsed.doc.title, "Heading Contract");
    assert_eq!(parsed.doc.id, "heading");
}

#[test]
fn extract_link_targets_from_occurrences_keeps_markdown_note_links() {
    let occurrences = vec![
        MarkdownTargetOccurrence {
            kind: MarkdownTargetOccurrenceKind::MarkdownLink,
            target: "docs/guide.md#intro".to_string(),
            surface: "[Guide](docs/guide.md#intro)".to_string(),
            byte_range: (0, 20),
            line_range: (1, 1),
        },
        MarkdownTargetOccurrence {
            kind: MarkdownTargetOccurrenceKind::MarkdownImage,
            target: "assets/logo.png".to_string(),
            surface: "![Image](assets/logo.png)".to_string(),
            byte_range: (21, 36),
            line_range: (2, 2),
        },
        MarkdownTargetOccurrence {
            kind: MarkdownTargetOccurrenceKind::WikiLink,
            target: "graph-c".to_string(),
            surface: "[[graph-c]]".to_string(),
            byte_range: (37, 44),
            line_range: (3, 3),
        },
    ];
    let first: &TargetOccurrenceCore<MarkdownTargetOccurrenceKind> = &occurrences[0];
    assert_eq!(first.target, "docs/guide.md#intro");
    let root = Path::new("/tmp/parser-doc");
    let path = Path::new("/tmp/parser-doc/adapter.md");

    let extracted = extract_link_targets_from_occurrences(&occurrences, path, root);

    assert_eq!(extracted.note_links, vec!["docs/guide", "graph-c"]);
    assert_eq!(extracted.attachments, vec!["assets/logo.png"]);
}

#[test]
fn parse_note_uses_parser_owned_target_occurrences_for_links_and_attachments() {
    let content = concat!(
        "# Target Contract\n\n",
        "[Guide](docs/guide.md#intro)\n",
        "![Image](assets/logo.png)\n",
        "![[IgnoredEmbed]]\n",
        "[Local](#local)\n",
        "[[graph-c]]\n",
    );
    let root = Path::new("/tmp/parser-doc");
    let path = Path::new("/tmp/parser-doc/adapter.md");

    let parsed =
        parse_note(path, root, content).unwrap_or_else(|| panic!("expected parsed note output"));

    assert_eq!(parsed.link_targets, vec!["docs/guide", "graph-c"]);
    assert_eq!(parsed.attachment_targets, vec!["assets/logo.png"]);
}
