use std::path::Path;

use xiuxian_wendao_parsers::{
    MarkdownSyntaxLintCode, lint_markdown_syntax, lint_markdown_syntax_with_path,
};

fn lint_with_required_frontmatter(body: &str) -> xiuxian_wendao_parsers::MarkdownSyntaxLintReport {
    let markdown = format!("---\ntitle: Demo\n---\n{body}");
    lint_markdown_syntax(markdown.as_str())
}

#[test]
fn lint_codes_classify_syntax_vs_repo_policy() {
    assert_eq!(
        MarkdownSyntaxLintCode::MixedWikilinkMarkdownLink.kind(),
        xiuxian_wendao_parsers::MarkdownLintKind::Syntax
    );
    assert_eq!(
        MarkdownSyntaxLintCode::MissingFrontmatter.kind(),
        xiuxian_wendao_parsers::MarkdownLintKind::AuthoringPolicy
    );
    assert_eq!(
        MarkdownSyntaxLintCode::BareObsidianWikilink.kind(),
        xiuxian_wendao_parsers::MarkdownLintKind::AuthoringPolicy
    );
}

#[test]
fn lint_reports_missing_frontmatter() {
    let report = lint_markdown_syntax("# Heading\nBody\n");
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::MissingFrontmatter
    );
}

#[test]
fn lint_reports_missing_frontmatter_title() {
    let report = lint_markdown_syntax("---\ntags: [demo]\n---\n# Heading\n");
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::MissingFrontmatterTitle
    );
}

#[test]
fn lint_reports_missing_skill_frontmatter_name() {
    let report = lint_markdown_syntax_with_path(
        Some(Path::new("skills/demo/SKILL.md")),
        "---\nmetadata:\n  version: \"1.0.0\"\n---\n# Skill\n",
    );
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::MissingSkillFrontmatterName
    );
}

#[test]
fn lint_reports_missing_skill_frontmatter_metadata() {
    let report = lint_markdown_syntax_with_path(
        Some(Path::new("skills/demo/SKILL.md")),
        "---\nname: demo-skill\n---\n# Skill\n",
    );
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::MissingSkillFrontmatterMetadata
    );
}

#[test]
fn lint_accepts_skill_md_with_strict_skill_frontmatter() {
    let report = lint_markdown_syntax_with_path(
        Some(Path::new("skills/demo/SKILL.md")),
        "---\nname: demo-skill\nmetadata:\n  version: \"1.0.0\"\n---\n# Skill\n",
    );
    assert!(report.is_clean(), "{report:#?}");
}

#[test]
fn lint_accepts_kind_marked_skill_doc_with_strict_skill_frontmatter() {
    let report = lint_markdown_syntax_with_path(
        Some(Path::new("docs/planner.md")),
        "---\nkind: SKILL.md\nname: planner\nmetadata:\n  version: \"1.0.0\"\n---\n# Planner\n",
    );
    assert!(report.is_clean(), "{report:#?}");
}

#[test]
fn lint_reports_invalid_frontmatter_yaml() {
    let report = lint_markdown_syntax("---\ntitle: demo\ntags: [alpha\n---\n# Heading\n");
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::InvalidFrontmatterYaml
    );
}

#[test]
fn lint_reports_unclosed_frontmatter() {
    let report = lint_markdown_syntax("---\ntitle: demo\n# Heading\n");
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::UnclosedFrontmatter
    );
}

#[test]
fn lint_reports_unclosed_fence() {
    let report = lint_with_required_frontmatter("# Heading\n```rust\nfn main() {}\n");
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].code, MarkdownSyntaxLintCode::UnclosedFence);
}

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

#[test]
fn lint_accepts_closed_frontmatter_and_fence() {
    let report =
        lint_markdown_syntax("---\ntitle: demo\n---\n# Heading\n```rust\nfn main() {}\n```\n");
    assert!(report.is_clean());
}

#[test]
fn lint_reports_frontmatter_position_from_document_lines() {
    let report = lint_markdown_syntax("---\na:\n  - 1\n  - [bad\n---\n# Heading\n");
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::InvalidFrontmatterYaml
    );
    assert!(report.issues[0].line >= 2);
}
