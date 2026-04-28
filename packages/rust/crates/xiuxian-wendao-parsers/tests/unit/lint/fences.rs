use super::lint_with_required_frontmatter;
use xiuxian_wendao_parsers::MarkdownSyntaxLintCode;

#[test]
fn lint_reports_unclosed_fence() {
    let report = lint_with_required_frontmatter("# Heading\n```rust\nfn main() {}\n");
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].code, MarkdownSyntaxLintCode::UnclosedFence);
}
