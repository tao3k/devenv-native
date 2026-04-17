mod fences;
mod frontmatter;
mod types;
mod wikilinks;

pub use types::{
    MarkdownLintKind, MarkdownSyntaxLintCode, MarkdownSyntaxLintIssue, MarkdownSyntaxLintReport,
};

/// Lint one Markdown document for lightweight syntax and authoring-policy failures.
///
/// The current lint surface is intentionally narrow and stable:
///
/// 1. unclosed YAML frontmatter
/// 2. invalid YAML frontmatter
/// 3. unclosed fenced code blocks
/// 4. repo-policy bare `[[target]]` wikilinks without explicit labels
/// 5. repo-policy redundant `[[target|target]]` wikilinks
/// 6. mixed `[[target]](label)` link syntax
/// 7. repo-policy non-canonical Obsidian alias order for target-like wikilinks
#[must_use]
pub fn lint_markdown_syntax(content: &str) -> MarkdownSyntaxLintReport {
    let mut issues = Vec::new();
    let body = frontmatter::analyze_frontmatter(content, &mut issues);
    fences::lint_fences(body.content, body.line_offset, &mut issues);
    wikilinks::lint_obsidian_wikilinks(body.content, body.line_offset, &mut issues);
    issues.sort_by_key(|issue| (issue.line, issue.column));
    MarkdownSyntaxLintReport { issues }
}
