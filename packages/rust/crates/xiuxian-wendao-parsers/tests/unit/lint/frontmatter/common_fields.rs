use xiuxian_wendao_parsers::{MarkdownSyntaxLintCode, lint_markdown_syntax};

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
        "saliency_base: 5.5\n",
        "decay_rate: 0.05\n",
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
        "saliency_base: 5.5\n",
        "decay_rate: 0.05\n",
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
        "saliency_base: 5.5\n",
        "decay_rate: 0.05\n",
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
        "saliency_base: 5.5\n",
        "decay_rate: 0.05\n",
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
        "saliency_base: 5.5\n",
        "decay_rate: 0.05\n",
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
        "saliency_base: 5.5\n",
        "decay_rate: 0.05\n",
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
        "saliency_base: 5.5\n",
        "decay_rate: 0.05\n",
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
