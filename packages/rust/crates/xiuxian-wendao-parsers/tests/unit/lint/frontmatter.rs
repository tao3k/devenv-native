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
fn lint_reports_missing_frontmatter_kind() {
    let report = lint_markdown_syntax(concat!(
        "---\n",
        "title: demo\n",
        "category: docs\n",
        "tags:\n",
        "  - demo\n",
        "description: Demo note\n",
        "author: xiuxian-artisan-workshop\n",
        "date: 2026-04-26T09:30-07:00\n",
        "metadata:\n",
        "  retrieval:\n",
        "    saliency_base: 5.5\n",
        "    decay_rate: 0.05\n",
        "---\n",
        "# Heading\n",
    ));
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::MissingFrontmatterKind
    );
    assert_eq!(
        report.issues[0].message,
        "frontmatter must include a non-empty `kind` field"
    );
}

#[test]
fn lint_reports_missing_frontmatter_category() {
    let report = lint_markdown_syntax(concat!(
        "---\n",
        "title: demo\n",
        "kind: reference\n",
        "tags:\n",
        "  - demo\n",
        "description: Demo note\n",
        "author: xiuxian-artisan-workshop\n",
        "date: 2026-04-26T09:30-07:00\n",
        "metadata:\n",
        "  retrieval:\n",
        "    saliency_base: 5.5\n",
        "    decay_rate: 0.05\n",
        "---\n",
        "# Heading\n",
    ));
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::MissingFrontmatterCategory
    );
}

#[test]
fn lint_reports_missing_frontmatter_tags() {
    let report = lint_markdown_syntax(concat!(
        "---\n",
        "title: demo\n",
        "kind: reference\n",
        "category: docs\n",
        "description: Demo note\n",
        "author: xiuxian-artisan-workshop\n",
        "date: 2026-04-26T09:30-07:00\n",
        "metadata:\n",
        "  retrieval:\n",
        "    saliency_base: 5.5\n",
        "    decay_rate: 0.05\n",
        "---\n",
        "# Heading\n",
    ));
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::MissingFrontmatterTags
    );
}

#[test]
fn lint_reports_missing_frontmatter_description() {
    let report = lint_markdown_syntax(concat!(
        "---\n",
        "title: demo\n",
        "kind: reference\n",
        "category: docs\n",
        "tags:\n",
        "  - demo\n",
        "author: xiuxian-artisan-workshop\n",
        "date: 2026-04-26T09:30-07:00\n",
        "metadata:\n",
        "  retrieval:\n",
        "    saliency_base: 5.5\n",
        "    decay_rate: 0.05\n",
        "---\n",
        "# Heading\n",
    ));
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::MissingFrontmatterDescription
    );
    assert_eq!(
        report.issues[0].message,
        "frontmatter must include a non-empty `description` field"
    );
}

#[test]
fn lint_reports_missing_frontmatter_author() {
    let report = lint_markdown_syntax(concat!(
        "---\n",
        "title: demo\n",
        "kind: reference\n",
        "category: docs\n",
        "tags:\n",
        "  - demo\n",
        "description: Demo note\n",
        "date: 2026-04-26T09:30-07:00\n",
        "metadata:\n",
        "  retrieval:\n",
        "    saliency_base: 5.5\n",
        "    decay_rate: 0.05\n",
        "---\n",
        "# Heading\n",
    ));
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::MissingFrontmatterAuthor
    );
    assert_eq!(
        report.issues[0].message,
        "frontmatter must include a non-empty `author` field"
    );
}

#[test]
fn lint_reports_missing_frontmatter_date() {
    let report = lint_markdown_syntax(concat!(
        "---\n",
        "title: demo\n",
        "kind: reference\n",
        "category: docs\n",
        "tags:\n",
        "  - demo\n",
        "description: Demo note\n",
        "author: xiuxian-artisan-workshop\n",
        "metadata:\n",
        "  retrieval:\n",
        "    saliency_base: 5.5\n",
        "    decay_rate: 0.05\n",
        "---\n",
        "# Heading\n",
    ));
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::MissingFrontmatterDate
    );
    assert_eq!(
        report.issues[0].message,
        "frontmatter must include a minute-precision `date` field"
    );
}

#[test]
fn lint_reports_invalid_frontmatter_date_precision() {
    let report = lint_markdown_syntax(concat!(
        "---\n",
        "title: demo\n",
        "kind: reference\n",
        "category: docs\n",
        "tags:\n",
        "  - demo\n",
        "description: Demo note\n",
        "author: xiuxian-artisan-workshop\n",
        "date: 2026-04-26T09:30:15-07:00\n",
        "metadata:\n",
        "  retrieval:\n",
        "    saliency_base: 5.5\n",
        "    decay_rate: 0.05\n",
        "---\n",
        "# Heading\n",
    ));
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::InvalidFrontmatterDatePrecision
    );
}

#[test]
fn lint_reports_missing_retrieval_entropy_fields() {
    let report = lint_markdown_syntax(concat!(
        "---\n",
        "title: demo\n",
        "kind: reference\n",
        "category: docs\n",
        "tags:\n",
        "  - demo\n",
        "description: Demo note\n",
        "author: xiuxian-artisan-workshop\n",
        "date: 2026-04-26T09:30-07:00\n",
        "---\n",
        "# Heading\n",
    ));
    assert_eq!(report.issues.len(), 2);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::MissingFrontmatterRetrievalSaliencyBase
    );
    assert_eq!(
        report.issues[1].code,
        MarkdownSyntaxLintCode::MissingFrontmatterRetrievalDecayRate
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
fn lint_accepts_closed_frontmatter_and_fence() {
    let report = lint_markdown_syntax(concat!(
        "---\n",
        "title: demo\n",
        "kind: reference\n",
        "category: docs\n",
        "tags:\n",
        "  - demo\n",
        "description: Demo note\n",
        "author: xiuxian-artisan-workshop\n",
        "date: 2026-04-26T09:30-07:00\n",
        "metadata:\n",
        "  retrieval:\n",
        "    saliency_base: 5.5\n",
        "    decay_rate: 0.05\n",
        "---\n",
        "# Heading\n",
        "```rust\n",
        "fn main() {}\n",
        "```\n",
    ));
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
