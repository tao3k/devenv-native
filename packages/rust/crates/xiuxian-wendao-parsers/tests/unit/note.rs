use xiuxian_wendao_parsers::document::MarkdownDocument;
use xiuxian_wendao_parsers::note::{
    NoteAggregate, fingerprint_markdown_note, fingerprint_markdown_symbol_surface,
    parse_markdown_note, parse_markdown_note_artifacts,
};
use xiuxian_wendao_parsers::references::MarkdownReference;
use xiuxian_wendao_parsers::sections::MarkdownSection;
use xiuxian_wendao_parsers::targets::MarkdownTargetOccurrence;

#[test]
fn parse_markdown_note_aggregates_document_sections_and_references() {
    let content = concat!(
        "---\n",
        "title: Aggregate Contract\n",
        "tags:\n",
        "  - parser\n",
        "---\n",
        "\n",
        "Body [Guide](docs/guide.md#intro).\n",
        "\n",
        "# Implementation\n",
        ":PROPERTIES:\n",
        ":ID: impl\n",
        ":END:\n",
        "\n",
        "See [[docs/spec.md|Spec]].\n",
    );

    let note = parse_markdown_note(content, "fallback");

    assert_eq!(note.document.core.title, "Aggregate Contract");
    assert_eq!(
        note.document.core.lead,
        "Body [Guide](docs/guide.md#intro)."
    );
    assert_eq!(note.document.core.tags, vec!["parser"]);
    assert_eq!(note.core.references.len(), 2);
    assert_eq!(note.core.targets.len(), 2);
    assert_eq!(note.core.sections.len(), 2);
    assert_eq!(note.core.sections[1].scope.heading_title, "Implementation");
    assert_eq!(
        note.core.sections[1]
            .metadata
            .attributes
            .get("ID")
            .map(String::as_str),
        Some("impl")
    );
}

#[test]
fn parse_markdown_note_keeps_markdown_link_targets_with_heading_bodies() {
    let content = concat!(
        "# Target Contract\n\n",
        "[Guide](docs/guide.md#intro)\n",
        "![Image](assets/logo.png)\n",
        "[[graph-c]]\n",
    );

    let note = parse_markdown_note(content, "fallback");

    assert_eq!(
        note.document.core.body,
        "# Target Contract\n\n[Guide](docs/guide.md#intro)\n![Image](assets/logo.png)\n[[graph-c]]\n"
    );
    assert_eq!(note.core.targets.len(), 3);
    assert_eq!(note.core.targets[0].target, "docs/guide.md#intro");
    assert_eq!(note.core.targets[1].target, "assets/logo.png");
    assert_eq!(note.core.targets[2].target, "graph-c");
}

#[test]
fn parse_markdown_note_wraps_markdown_items_in_shared_note_core() {
    let note = parse_markdown_note("[Doc](docs/guide.md)\n\n# H\n", "fallback");
    let aggregate: &NoteAggregate<
        MarkdownDocument,
        MarkdownReference,
        MarkdownTargetOccurrence,
        MarkdownSection,
    > = &note;

    assert_eq!(note.core.references.len(), 1);
    assert_eq!(note.core.targets.len(), 1);
    assert_eq!(note.core.sections.len(), 2);
    assert_eq!(aggregate.document.core.title, "H");
}

#[test]
fn markdown_note_fingerprint_ignores_layout_only_body_churn() {
    let base = parse_markdown_note(
        "# Alpha\n\nAlpha body.\n\n## Overview\n\nAlpha section.\n",
        "fallback",
    );
    let layout_only = parse_markdown_note(
        "# Alpha\n\nAlpha body.\n\n\n## Overview\n\nAlpha section.\n\n",
        "fallback",
    );

    assert_eq!(
        fingerprint_markdown_note(&base),
        fingerprint_markdown_note(&layout_only)
    );
}

#[test]
fn markdown_note_fingerprint_invalidates_on_semantic_note_change() {
    let base = parse_markdown_note(
        "# Alpha\n\nAlpha body.\n\n## Overview\n\nAlpha section.\n",
        "fallback",
    );
    let changed = parse_markdown_note(
        "# Alpha\n\nBeta body.\n\n## Overview\n\nAlpha section.\n",
        "fallback",
    );

    assert_ne!(
        fingerprint_markdown_note(&base),
        fingerprint_markdown_note(&changed)
    );
}

#[test]
fn markdown_symbol_fingerprint_ignores_layout_only_body_churn() {
    let base = parse_markdown_note(
        concat!(
            "# Alpha\n\n",
            "Body text.\n\n",
            "- [ ] Ship parser lane\n\n",
            "## Overview\n",
            ":PROPERTIES:\n",
            ":ID: alpha\n",
            ":OBSERVE: lang:rust \"fn $NAME()\"\n",
            ":END:\n",
        ),
        "fallback",
    );
    let layout_only = parse_markdown_note(
        concat!(
            "# Alpha\n\n",
            "Body text with extra paragraph.\n\n",
            "\n",
            "- [ ] Ship parser lane\n\n",
            "## Overview\n",
            ":PROPERTIES:\n",
            ":ID: alpha\n",
            ":OBSERVE: lang:rust \"fn $NAME()\"\n",
            ":END:\n\n",
        ),
        "fallback",
    );

    assert_eq!(
        fingerprint_markdown_symbol_surface(&base),
        fingerprint_markdown_symbol_surface(&layout_only)
    );
}

#[test]
fn markdown_symbol_fingerprint_invalidates_on_symbol_surface_change() {
    let base = parse_markdown_note(
        concat!(
            "# Alpha\n\n",
            "- [ ] Ship parser lane\n\n",
            "## Overview\n",
            ":PROPERTIES:\n",
            ":ID: alpha\n",
            ":END:\n",
        ),
        "fallback",
    );
    let changed = parse_markdown_note(
        concat!(
            "# Alpha\n\n",
            "- [ ] Ship parser lane\n\n",
            "## Implementation\n",
            ":PROPERTIES:\n",
            ":ID: beta\n",
            ":END:\n",
        ),
        "fallback",
    );

    assert_ne!(
        fingerprint_markdown_symbol_surface(&base),
        fingerprint_markdown_symbol_surface(&changed)
    );
}

#[test]
fn parse_markdown_note_artifacts_match_standalone_note_and_symbol_fingerprint() {
    let content = concat!(
        "# Alpha\n\n",
        "- [ ] Ship parser lane\n\n",
        "## Overview\n",
        ":PROPERTIES:\n",
        ":ID: alpha\n",
        ":OBSERVE: lang:rust \"fn $NAME()\"\n",
        ":END:\n",
    );

    let standalone_note = parse_markdown_note(content, "fallback");
    let standalone_symbol = fingerprint_markdown_symbol_surface(&standalone_note);
    let artifacts = parse_markdown_note_artifacts(content, "fallback");

    assert_eq!(artifacts.note, standalone_note);
    assert_eq!(artifacts.symbol_fingerprint, standalone_symbol);
}

#[test]
fn parse_markdown_note_uses_structural_fallback_title_not_code_fence_text() {
    let note = parse_markdown_note(
        concat!("```md\n", "# Not a heading\n", "```\n",),
        "fallback",
    );

    assert_eq!(note.document.core.title, "fallback");
    assert!(note.document.core.lead.is_empty());
    assert!(
        note.core
            .sections
            .iter()
            .all(|section| section.heading_title().is_empty())
    );
}
