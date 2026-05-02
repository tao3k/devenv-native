//! Markdown lint path discovery and transient directory filtering.

mod config;
mod files;

use super::MarkdownLintArgs;

pub(crate) use files::{collect_markdown_files, display_path, first_transient_repo_dir};
