use super::facts::DiagnosticFacts;
use super::link::{LinkIssueContext, TargetMetadata, split_target_path_and_heading};
use super::text::normalize_hint;
use crate::lint::{MarkdownLintIssue, contract::diagnostic_contract};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use xiuxian_wendao_parsers::{MarkdownSyntaxLintIssue, parse_markdown_document};

pub(in crate::lint) struct DiagnosticContext<'a> {
    root: &'a Path,
    title_cache: HashMap<PathBuf, Option<String>>,
}

impl<'a> DiagnosticContext<'a> {
    pub(in crate::lint) fn new(root: &'a Path) -> Self {
        Self {
            root,
            title_cache: HashMap::new(),
        }
    }

    pub(in crate::lint) fn build_issue(
        &mut self,
        source_path: &Path,
        markdown: &str,
        issue: MarkdownSyntaxLintIssue,
    ) -> MarkdownLintIssue {
        let source = source_line(markdown, issue.line).map(ToOwned::to_owned);
        let link = source
            .as_deref()
            .and_then(|line| LinkIssueContext::from_source(issue.code, line, issue.column));
        let target_metadata = link
            .as_ref()
            .map(|context| self.resolve_target_metadata(source_path, context.target.as_str()));
        let duplicates_heading = link
            .as_ref()
            .and_then(|context| context.label.as_deref())
            .zip(
                target_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.heading.as_deref()),
            )
            .is_some_and(|(label, heading)| normalize_hint(label) == normalize_hint(heading));
        let facts = DiagnosticFacts::from_parser_issue(
            issue,
            source,
            link,
            target_metadata,
            duplicates_heading,
        );
        diagnostic_contract().render_issue(&facts)
    }

    fn resolve_target_metadata(&mut self, source_path: &Path, raw_target: &str) -> TargetMetadata {
        let (path_target, heading) = split_target_path_and_heading(raw_target);
        let title = self
            .resolve_target_path(source_path, path_target.as_str())
            .and_then(|path| self.read_title(path.as_path()));
        TargetMetadata {
            raw: raw_target.trim().to_string(),
            heading,
            title,
        }
    }

    fn resolve_target_path(&self, source_path: &Path, raw_target: &str) -> Option<PathBuf> {
        let trimmed = raw_target
            .trim()
            .trim_start_matches('/')
            .trim_end_matches('/');
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with("id:")
            || trimmed.starts_with("obsidian://")
            || trimmed.starts_with("wendao://")
        {
            return None;
        }
        let target = Path::new(trimmed);
        for base in candidate_base_dirs(source_path, self.root) {
            for candidate in candidate_target_paths(base.as_path(), target) {
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        None
    }

    fn read_title(&mut self, path: &Path) -> Option<String> {
        if let Some(cached) = self.title_cache.get(path) {
            return cached.clone();
        }

        let title = std::fs::read_to_string(path)
            .ok()
            .map(|content| {
                let fallback = path
                    .file_stem()
                    .and_then(std::ffi::OsStr::to_str)
                    .unwrap_or("document");
                parse_markdown_document(&content, fallback)
                    .core
                    .title
                    .trim()
                    .to_string()
            })
            .filter(|title| !title.is_empty());
        self.title_cache.insert(path.to_path_buf(), title.clone());
        title
    }
}

fn source_line(markdown: &str, line: usize) -> Option<&str> {
    markdown.lines().nth(line.saturating_sub(1))
}

fn candidate_base_dirs(source_path: &Path, root: &Path) -> Vec<PathBuf> {
    let source_dir = source_path
        .parent()
        .map_or_else(|| root.to_path_buf(), Path::to_path_buf);
    let mut bases = Vec::new();
    let mut cursor = Some(source_dir.as_path());
    while let Some(path) = cursor {
        bases.push(path.to_path_buf());
        if path == root {
            break;
        }
        cursor = path.parent();
    }
    if !bases.iter().any(|path| path == root) {
        bases.push(root.to_path_buf());
    }
    bases
}

fn candidate_target_paths(base: &Path, target: &Path) -> Vec<PathBuf> {
    if target.extension().is_some() {
        vec![base.join(target)]
    } else {
        vec![
            base.join(target),
            base.join(target).with_extension("md"),
            base.join(target).join("index.md"),
        ]
    }
}
