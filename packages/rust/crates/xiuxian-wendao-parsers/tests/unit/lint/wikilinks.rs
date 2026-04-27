use super::lint_with_required_frontmatter;
use xiuxian_wendao_parsers::MarkdownSyntaxLintCode;

#[test]
fn lint_reports_mixed_wikilink_markdown_link_syntax() {
    let report = lint_with_required_frontmatter("# Heading\nSee [[docs/index]](Index).\n");
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::MixedWikilinkMarkdownLink
    );
}

#[test]
fn lint_reports_bare_wikilinks_without_explicit_labels() {
    let report = lint_with_required_frontmatter("# Heading\nSee [[docs/index]].\n");
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::BareObsidianWikilink
    );
}

#[test]
fn lint_reports_bare_local_address_wikilinks() {
    let report = lint_with_required_frontmatter("# Heading\nSee [[#Implementation]].\n");
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::BareObsidianWikilink
    );
}

#[test]
fn lint_reports_redundant_obsidian_labels() {
    let report = lint_with_required_frontmatter("# Heading\nSee [[docs/index|docs/index]].\n");
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::RedundantObsidianLabel
    );
}

#[test]
fn lint_reports_redundant_heading_display_labels() {
    let report = lint_with_required_frontmatter(
        "# Heading\nSee [[docs/index#Parser Contracts|Parser Contracts]].\n",
    );
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::RedundantObsidianLabel
    );
}

#[test]
fn lint_reports_reversed_obsidian_alias_order_for_path_targets() {
    let report = lint_with_required_frontmatter(
        "# Heading\nSee [[Docs Maintenance Playbook|01_core/106_docs_maintenance_playbook]].\n",
    );
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::NonCanonicalObsidianAliasOrder
    );
}

#[test]
fn lint_accepts_official_obsidian_aliases_for_spaced_targets_and_subheadings() {
    let report = lint_with_required_frontmatter(concat!(
        "# Heading\n",
        "See [[Three laws of motion|Overview]].\n",
        "And [[Three laws of motion.md|Laws Note]].\n",
        "And [[Help and support#Questions and advice#Report bugs and request features|Bug Reports]].\n",
        "And [[#^local-block-id|Local Block Context]].\n",
    ));
    assert!(report.is_clean(), "{report:#?}");
}

#[test]
fn lint_skips_wikilink_examples_inside_code_and_embeds() {
    let report = lint_with_required_frontmatter(
        "# Heading\n`[[docs/index]]`\n\n`[[Docs Maintenance Playbook|01_core/106_docs_maintenance_playbook]]`\n\n![[files/spec.pdf|100]]\n",
    );
    assert!(report.is_clean());
}

#[test]
fn lint_accepts_official_obsidian_heading_and_block_link_shapes() {
    let report = lint_with_required_frontmatter(
        "# Heading\nSee [[docs/index#Parser Contracts|Contracts Overview]].\nAnd [[#Local Heading|Local Heading Context]].\nAnd [[docs/index#^block-id|Block Context]].\n",
    );
    assert!(report.is_clean());
}
