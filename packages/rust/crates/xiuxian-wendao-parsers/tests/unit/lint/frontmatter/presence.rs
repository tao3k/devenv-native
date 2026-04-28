use xiuxian_wendao_parsers::{MarkdownSyntaxLintCode, lint_markdown_syntax};

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
    let codes = report
        .issues
        .iter()
        .map(|issue| issue.code)
        .collect::<Vec<_>>();
    assert_eq!(
        codes,
        vec![
            MarkdownSyntaxLintCode::MissingFrontmatterTitle,
            MarkdownSyntaxLintCode::MissingFrontmatterKind,
            MarkdownSyntaxLintCode::MissingFrontmatterCategory,
            MarkdownSyntaxLintCode::MissingFrontmatterDescription,
            MarkdownSyntaxLintCode::MissingFrontmatterAuthor,
            MarkdownSyntaxLintCode::MissingFrontmatterDate,
            MarkdownSyntaxLintCode::MissingFrontmatterRetrievalSaliencyBase,
            MarkdownSyntaxLintCode::MissingFrontmatterRetrievalDecayRate,
        ]
    );
}

#[test]
fn lint_reports_non_mapping_frontmatter_missing_common_fields() {
    let report = lint_markdown_syntax("---\n- demo\n---\n# Heading\n");
    assert_eq!(report.issues.len(), 9);
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
fn lint_reports_frontmatter_position_from_document_lines() {
    let report = lint_markdown_syntax("---\na:\n  - 1\n  - [bad\n---\n# Heading\n");
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::InvalidFrontmatterYaml
    );
    assert!(report.issues[0].line >= 2);
}
