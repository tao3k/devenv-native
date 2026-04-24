use xiuxian_wendao_parsers::targets::{
    MarkdownTargetOccurrenceKind, TargetOccurrenceCore, extract_targets,
};

#[test]
fn extract_targets_preserves_links_and_images_in_document_order() {
    let markdown = r"
Body [Guide](docs/guide.md#intro) and [[docs/spec.md|Spec]].

![Image](assets/logo.png)
![[IgnoredEmbed]]
[[graph-c]]
[Local](#local-section)
";

    let targets = extract_targets(markdown);
    assert_eq!(targets.len(), 6);

    let first: &TargetOccurrenceCore<MarkdownTargetOccurrenceKind> = &targets[0];
    assert_eq!(first.target, "docs/guide.md#intro");
    assert_eq!(first.surface, "[Guide](docs/guide.md#intro)");
    assert_eq!(first.line_range, (2, 2));

    assert_eq!(targets[0].kind, MarkdownTargetOccurrenceKind::MarkdownLink);
    assert_eq!(targets[0].target, "docs/guide.md#intro");
    assert_eq!(targets[0].surface, "[Guide](docs/guide.md#intro)");

    assert_eq!(targets[1].kind, MarkdownTargetOccurrenceKind::WikiLink);
    assert_eq!(targets[1].target, "docs/spec.md");
    assert_eq!(targets[1].surface, "[[docs/spec.md|Spec]]");
    assert_eq!(targets[1].line_range, (2, 2));

    assert_eq!(targets[2].kind, MarkdownTargetOccurrenceKind::MarkdownImage);
    assert_eq!(targets[2].target, "assets/logo.png");
    assert_eq!(targets[2].surface, "![Image](assets/logo.png)");
    assert_eq!(targets[2].line_range, (4, 4));

    assert_eq!(targets[3].kind, MarkdownTargetOccurrenceKind::WikiEmbed);
    assert_eq!(targets[3].target, "IgnoredEmbed");
    assert_eq!(targets[3].surface, "![[IgnoredEmbed]]");
    assert_eq!(targets[3].line_range, (5, 5));

    assert_eq!(targets[4].kind, MarkdownTargetOccurrenceKind::WikiLink);
    assert_eq!(targets[4].target, "graph-c");
    assert_eq!(targets[4].surface, "[[graph-c]]");
    assert_eq!(targets[4].line_range, (6, 6));

    assert_eq!(targets[5].kind, MarkdownTargetOccurrenceKind::MarkdownLink);
    assert_eq!(targets[5].target, "#local-section");
    assert_eq!(targets[5].surface, "[Local](#local-section)");
    assert_eq!(targets[5].line_range, (7, 7));
}

#[test]
fn extract_targets_keeps_dot_prefixed_markdown_paths() {
    let markdown = r"
[Dot](.data/internal.md#Stable Heading)
[DotSlash](./.data/internal.md#Stable Heading)
![Attachment](.cache/rendered.png)
";

    let targets = extract_targets(markdown);
    assert_eq!(targets.len(), 3);

    assert_eq!(targets[0].kind, MarkdownTargetOccurrenceKind::MarkdownLink);
    assert_eq!(targets[0].target, ".data/internal.md#Stable Heading");
    assert_eq!(
        targets[0].surface,
        "[Dot](.data/internal.md#Stable Heading)"
    );

    assert_eq!(targets[1].kind, MarkdownTargetOccurrenceKind::MarkdownLink);
    assert_eq!(targets[1].target, "./.data/internal.md#Stable Heading");
    assert_eq!(
        targets[1].surface,
        "[DotSlash](./.data/internal.md#Stable Heading)"
    );

    assert_eq!(targets[2].kind, MarkdownTargetOccurrenceKind::MarkdownImage);
    assert_eq!(targets[2].target, ".cache/rendered.png");
    assert_eq!(targets[2].surface, "![Attachment](.cache/rendered.png)");
}

#[test]
fn extract_targets_skips_markdown_target_literals_inside_code() {
    let markdown = concat!(
        "`[Inline](.data/inline.md#Heading)`\n\n",
        "```md\n",
        "[Fence](.data/fence.md#Heading)\n",
        "![Image](.cache/rendered.png)\n",
        "```\n",
    );

    let targets = extract_targets(markdown);
    assert!(targets.is_empty(), "{targets:#?}");
}
