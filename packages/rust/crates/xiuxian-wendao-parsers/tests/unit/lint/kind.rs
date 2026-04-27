use xiuxian_wendao_parsers::{MarkdownLintKind, MarkdownSyntaxLintCode};

#[test]
fn lint_codes_classify_syntax_vs_repo_policy() {
    assert_eq!(
        MarkdownSyntaxLintCode::MixedWikilinkMarkdownLink.kind(),
        MarkdownLintKind::Syntax
    );
    assert_eq!(
        MarkdownSyntaxLintCode::MissingFrontmatter.kind(),
        MarkdownLintKind::AuthoringPolicy
    );
    assert_eq!(
        MarkdownSyntaxLintCode::BareObsidianWikilink.kind(),
        MarkdownLintKind::AuthoringPolicy
    );
}
