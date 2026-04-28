use std::path::Path;

use xiuxian_wendao_parsers::{MarkdownSyntaxLintReport, lint_markdown_syntax_with_path};

mod metadata;
mod required;
mod valid;

fn lint_skill(markdown: &str) -> MarkdownSyntaxLintReport {
    lint_markdown_syntax_with_path(Some(Path::new("skills/demo/SKILL.md")), markdown)
}

fn lint_doc(path: &str, markdown: &str) -> MarkdownSyntaxLintReport {
    lint_markdown_syntax_with_path(Some(Path::new(path)), markdown)
}
