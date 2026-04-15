use serde_yaml::Value;
use xiuxian_wendao_parsers::document::{DocumentEnvelope, DocumentFormat, parse_markdown_document};

#[test]
fn parse_markdown_document_extracts_frontmatter_and_body_metadata() {
    let content = concat!(
        "---\n",
        "title: Parser Contract\n",
        "type: design\n",
        "tags:\n",
        "  - rust\n",
        "  - parser\n",
        "---\n",
        "\n",
        "First paragraph line.\n",
        "Second paragraph line.\n",
    );

    let document = parse_markdown_document(content, "fallback");

    assert_eq!(document.core.format, DocumentFormat::Markdown);
    assert_eq!(document.core.title, "Parser Contract");
    assert_eq!(document.core.tags, vec!["parser", "rust"]);
    assert_eq!(document.core.doc_type.as_deref(), Some("design"));
    assert_eq!(
        document.core.lead,
        "First paragraph line. Second paragraph line."
    );
    assert_eq!(document.core.word_count, 6);
    assert_eq!(
        document.core.body,
        "First paragraph line.\nSecond paragraph line.\n"
    );
    assert!(document.raw_metadata.is_some());
}

#[test]
fn parse_markdown_document_uses_heading_then_fallback_title() {
    let heading_document = parse_markdown_document("# Heading Title\n\nBody text.\n", "fallback");
    assert_eq!(heading_document.core.title, "Heading Title");

    let fallback_document = parse_markdown_document("Body only.\n", "fallback");
    assert_eq!(fallback_document.core.title, "fallback");
}

#[test]
fn parse_markdown_document_uses_structural_title_and_lead_not_code_fence_text() {
    let document = parse_markdown_document(
        concat!("```md\n", "# Not a heading\n", "Body text.\n", "```\n",),
        "fallback",
    );

    assert_eq!(document.core.title, "fallback");
    assert!(document.core.lead.is_empty());
}

#[test]
fn parse_markdown_document_deduplicates_top_level_tags() {
    let content = concat!(
        "---\n",
        "tags:\n",
        "  - search\n",
        "  - vector\n",
        "  - search\n",
        "---\n",
        "Body.\n",
    );

    let document = parse_markdown_document(content, "fallback");
    assert_eq!(document.core.tags, vec!["search", "vector"]);
}

#[test]
fn parse_markdown_document_wraps_shared_document_envelope() {
    let document = parse_markdown_document("---\ntitle: Envelope\n---\nBody.\n", "fallback");
    let envelope: &DocumentEnvelope<Value> = &document;

    assert!(envelope.raw_metadata.is_some());
    assert_eq!(envelope.core.title, "Envelope");
}
