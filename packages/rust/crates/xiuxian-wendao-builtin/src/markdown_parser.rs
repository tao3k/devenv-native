//! Builtin Markdown parser repo-intelligence plugin.

use std::fs;
use std::path::{Path, PathBuf};

use xiuxian_wendao_core::repo_intelligence::{
    AnalysisContext, DocRecord, PluginAnalysisOutput, PluginRegistry, RepoIntelligenceError,
    RepoIntelligencePlugin, RepositoryAnalysisOutput, RepositoryRecord,
};

const MARKDOWN_PARSER_PLUGIN_ID: &str = "markdown-parser";

struct MarkdownParserPlugin;

fn register_markdown_parser_into(
    registry: &mut PluginRegistry,
) -> Result<(), RepoIntelligenceError> {
    registry.register(MarkdownParserPlugin)
}

inventory::submit! {
    xiuxian_wendao_core::repo_intelligence::BuiltinPluginRegistrar::new(
        MARKDOWN_PARSER_PLUGIN_ID,
        register_markdown_parser_into,
    )
}

impl RepoIntelligencePlugin for MarkdownParserPlugin {
    fn id(&self) -> &'static str {
        MARKDOWN_PARSER_PLUGIN_ID
    }

    fn supports_repository(
        &self,
        repository: &xiuxian_wendao_core::repo_intelligence::RegisteredRepository,
    ) -> bool {
        repository
            .plugins
            .iter()
            .any(|plugin| plugin.id() == MARKDOWN_PARSER_PLUGIN_ID)
    }

    fn analyze_file(
        &self,
        context: &AnalysisContext,
        file: &xiuxian_wendao_core::repo_intelligence::RepoSourceFile,
    ) -> Result<PluginAnalysisOutput, RepoIntelligenceError> {
        if !is_markdown_path(file.path.as_str()) {
            return Ok(PluginAnalysisOutput::default());
        }
        Ok(PluginAnalysisOutput {
            docs: vec![doc_record(
                context.repository.id.as_str(),
                file.path.as_str(),
                file.contents.as_str(),
            )],
            ..PluginAnalysisOutput::default()
        })
    }

    fn analyze_repository(
        &self,
        context: &AnalysisContext,
        repository_root: &Path,
    ) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
        Ok(RepositoryAnalysisOutput {
            repository: Some(RepositoryRecord::from(&context.repository)),
            docs: discover_markdown_files(context.repository.id.as_str(), repository_root)?,
            ..RepositoryAnalysisOutput::default()
        })
    }
}

fn discover_markdown_files(
    repo_id: &str,
    repository_root: &Path,
) -> Result<Vec<DocRecord>, RepoIntelligenceError> {
    let mut paths = Vec::new();
    collect_markdown_paths(repository_root, repository_root, &mut paths)?;
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let relative = relative_path(repository_root, path.as_path())?;
            let contents = fs::read_to_string(path.as_path()).map_err(|error| {
                RepoIntelligenceError::AnalysisFailed {
                    message: format!("failed to read Markdown file `{}`: {error}", path.display()),
                }
            })?;
            Ok(doc_record(repo_id, relative.as_str(), contents.as_str()))
        })
        .collect()
}

fn collect_markdown_paths(
    repository_root: &Path,
    current: &Path,
    paths: &mut Vec<PathBuf>,
) -> Result<(), RepoIntelligenceError> {
    let entries = fs::read_dir(current).map_err(|error| RepoIntelligenceError::AnalysisFailed {
        message: format!("failed to read `{}`: {error}", current.display()),
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| RepoIntelligenceError::AnalysisFailed {
            message: format!("failed to enumerate `{}`: {error}", current.display()),
        })?;
        let path = entry.path();
        let file_name = entry.file_name();
        if file_name
            .to_str()
            .is_some_and(|name| name.starts_with('.') || matches!(name, "target" | "node_modules"))
        {
            continue;
        }
        let file_type =
            entry
                .file_type()
                .map_err(|error| RepoIntelligenceError::AnalysisFailed {
                    message: format!("failed to inspect `{}`: {error}", path.display()),
                })?;
        if file_type.is_dir() {
            collect_markdown_paths(repository_root, path.as_path(), paths)?;
        } else if file_type.is_file()
            && relative_path(repository_root, path.as_path())
                .ok()
                .is_some_and(|relative| is_markdown_path(relative.as_str()))
        {
            paths.push(path);
        }
    }
    Ok(())
}

fn is_markdown_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let lower = normalized.to_ascii_lowercase();
    lower.ends_with(".md") || lower.ends_with(".markdown")
}

fn doc_record(repo_id: &str, relative_path: &str, contents: &str) -> DocRecord {
    DocRecord {
        repo_id: repo_id.to_string(),
        doc_id: format!("repo:{repo_id}:doc:{relative_path}"),
        title: markdown_title(relative_path, contents),
        path: relative_path.to_string(),
        format: Some(markdown_format(relative_path).to_string()),
        doc_target: None,
    }
}

fn markdown_title(relative_path: &str, contents: &str) -> String {
    contents
        .lines()
        .find_map(markdown_heading_title)
        .unwrap_or_else(|| {
            Path::new(relative_path)
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("doc")
                .to_string()
        })
}

fn markdown_heading_title(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let marker_count = trimmed
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if !(1..=6).contains(&marker_count) {
        return None;
    }
    let title = trimmed.get(marker_count..)?.trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}

fn markdown_format(relative_path: &str) -> &'static str {
    if relative_path.to_ascii_lowercase().ends_with(".markdown") {
        "markdown"
    } else {
        "md"
    }
}

fn relative_path(root: &Path, path: &Path) -> Result<String, RepoIntelligenceError> {
    let relative =
        path.strip_prefix(root)
            .map_err(|error| RepoIntelligenceError::AnalysisFailed {
                message: format!(
                    "failed to compute relative path for `{}` against `{}`: {error}",
                    path.display(),
                    root.display()
                ),
            })?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}
