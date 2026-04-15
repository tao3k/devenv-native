use xiuxian_wendao_parsers::toc::{MarkdownTocDocument, TocDocument, parse_markdown_toc};

#[test]
fn parse_markdown_toc_aggregates_document_and_sections() {
    let content = concat!(
        "---\n",
        "title: Aggregate Contract\n",
        "tags:\n",
        "  - parser\n",
        "---\n",
        "\n",
        "Body text.\n",
        "\n",
        "# Implementation\n",
        ":PROPERTIES:\n",
        ":ID: impl\n",
        ":END:\n",
        "\n",
        "Section body.\n",
    );

    let toc = parse_markdown_toc(content, "fallback");

    assert_eq!(toc.document.core.title, "Aggregate Contract");
    assert_eq!(toc.document.core.lead, "Body text.");
    assert_eq!(toc.document.core.tags, vec!["parser"]);
    assert_eq!(toc.sections.len(), 2);
    assert_eq!(toc.sections[1].scope.heading_title, "Implementation");
    assert_eq!(
        toc.sections[1]
            .metadata
            .attributes
            .get("ID")
            .map(String::as_str),
        Some("impl")
    );
}

#[test]
fn parse_markdown_toc_wraps_markdown_items_in_shared_toc_core() {
    let toc = parse_markdown_toc("# Heading\n", "fallback");
    let aggregate: &TocDocument<_, _> = &toc;
    let markdown: &MarkdownTocDocument = &toc;

    assert_eq!(aggregate.document.core.title, "Heading");
    assert_eq!(markdown.sections.len(), 1);
}

#[test]
fn parse_markdown_toc_ignores_code_fence_heading_like_lines() {
    let toc = parse_markdown_toc(
        concat!("# Root\n\n", "```md\n", "## Not a heading\n", "```\n",),
        "fallback",
    );

    assert_eq!(toc.sections.len(), 1);
    assert_eq!(toc.sections[0].heading_title(), "Root");
}

#[test]
fn parse_markdown_toc_uses_structural_fallback_title_not_code_fence_text() {
    let toc = parse_markdown_toc(
        concat!("```md\n", "# Not a heading\n", "```\n",),
        "fallback",
    );

    assert_eq!(toc.document.core.title, "fallback");
    assert!(toc.document.core.lead.is_empty());
    assert!(
        toc.sections
            .iter()
            .all(|section| section.heading_title().is_empty())
    );
}
