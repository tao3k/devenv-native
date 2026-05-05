//! Public Markdown syntax lint entry points.

use std::path::Path;

use super::types::MarkdownSyntaxLintReport;
use super::{fences, frontmatter, wikilinks};

/// Lint one Markdown document for lightweight syntax and authoring-policy failures.
///
/// The current lint surface is intentionally narrow and stable:
///
/// 1. required YAML frontmatter presence
/// 2. required primary frontmatter fields
///    - ordinary Markdown documents require top-level identity, provenance,
///      tags, and retrieval-hint fields
///    - ordinary Markdown `metadata` is extension space and is not required to
///      mirror top-level frontmatter fields
///    - `SKILL.md` or `kind: SKILL.md` documents must satisfy the
///      parser-owned SKILL.md frontmatter contract
/// 3. unclosed YAML frontmatter
/// 4. invalid YAML frontmatter
/// 5. unclosed fenced code blocks
/// 6. repo-policy bare `[[target]]` wikilinks without explicit labels
/// 7. repo-policy redundant `[[target|target]]` wikilinks
/// 8. mixed `[[target]](label)` link syntax
/// 9. repo-policy non-canonical Obsidian alias order for target-like wikilinks
#[must_use]
pub fn lint_markdown_syntax(content: &str) -> MarkdownSyntaxLintReport {
    lint_markdown_syntax_with_path(None, content)
}

/// Lint one Markdown document with optional source-path context for
/// path-sensitive frontmatter variants such as `SKILL.md`.
#[must_use]
pub fn lint_markdown_syntax_with_path(
    path: Option<&Path>,
    content: &str,
) -> MarkdownSyntaxLintReport {
    let mut issues = Vec::new();
    let body = frontmatter::analyze_frontmatter(path, content, &mut issues);
    fences::lint_fences(body.content, body.line_offset, &mut issues);
    wikilinks::lint_obsidian_wikilinks(body.content, body.line_offset, &mut issues);
    issues.sort_by_key(|issue| (issue.line, issue.column));
    MarkdownSyntaxLintReport { issues }
}
