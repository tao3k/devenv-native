//! Parser-owned Markdown syntax lint orchestration.

mod api;
mod fences;
mod frontmatter;
mod skill_frontmatter;
mod types;
mod wikilinks;

pub use api::{lint_markdown_syntax, lint_markdown_syntax_with_path};
pub use types::{
    MarkdownLintKind, MarkdownSyntaxLintCode, MarkdownSyntaxLintIssue, MarkdownSyntaxLintReport,
};
