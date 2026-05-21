//! Configured Markdown corpus audit for `SearchStrategyFlow` real scenarios.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

use super::discovery::{
    discover_search_strategy_flow_candidate_inputs,
    discover_search_strategy_flow_candidate_inputs_with_limit, heading_sections,
    is_ignored_walk_entry, is_markdown_path, repo_relative_path,
    search_strategy_flow_candidate_input_batch,
};
use super::types::{
    MARKDOWN_HEADING_CANDIDATE_SOURCE, SearchStrategyFlowCandidateInput,
    SearchStrategyFlowCandidateInputBatch,
};

const ROOT_WENDAO_CONFIG_SURFACE: &str = "root-wendao.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchStrategyFlowConfiguredMarkdownCorpusAudit {
    pub(crate) config_surface: String,
    pub(crate) configured_project_count: usize,
    pub(crate) include_dir_count: usize,
    pub(crate) markdown_file_count: usize,
    pub(crate) heading_count: usize,
    pub(crate) rows: Vec<SearchStrategyFlowConfiguredMarkdownCorpusRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchStrategyFlowConfiguredMarkdownCorpusRow {
    pub(crate) include_dir: String,
    pub(crate) markdown_file_count: usize,
    pub(crate) heading_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchStrategyFlowConfiguredMarkdownReplayFamily {
    pub(crate) include_dir: String,
    pub(crate) markdown_file_count: usize,
    pub(crate) heading_count: usize,
    pub(crate) batch: SearchStrategyFlowCandidateInputBatch,
}

#[derive(Debug, Clone)]
struct ConfiguredMarkdownCorpusScan {
    row: SearchStrategyFlowConfiguredMarkdownCorpusRow,
    files: Vec<ConfiguredMarkdownCorpusFile>,
}

#[derive(Debug, Clone)]
struct ConfiguredMarkdownCorpusFile {
    relative_path: String,
    heading_count: usize,
}

pub(crate) fn audit_configured_search_strategy_flow_markdown_corpus(
    project_root: &Path,
) -> Result<SearchStrategyFlowConfiguredMarkdownCorpusAudit, String> {
    let (configured_project_count, scans) = configured_markdown_corpus_scans(project_root)?;
    let rows = scans
        .iter()
        .map(|scan| scan.row.clone())
        .collect::<Vec<_>>();
    let unique_heading_counts = unique_configured_heading_counts(&scans);

    Ok(SearchStrategyFlowConfiguredMarkdownCorpusAudit {
        config_surface: ROOT_WENDAO_CONFIG_SURFACE.to_owned(),
        configured_project_count,
        include_dir_count: rows.len(),
        markdown_file_count: unique_heading_counts.len(),
        heading_count: unique_heading_counts.values().sum(),
        rows,
    })
}

pub(crate) fn configured_search_strategy_flow_markdown_replay_families(
    project_root: &Path,
    intent: &str,
) -> Result<Vec<SearchStrategyFlowConfiguredMarkdownReplayFamily>, String> {
    configured_search_strategy_flow_markdown_replay_families_with_limit(project_root, intent, None)
}

pub(crate) fn configured_search_strategy_flow_markdown_replay_families_with_limit(
    project_root: &Path,
    intent: &str,
    max_candidates: Option<usize>,
) -> Result<Vec<SearchStrategyFlowConfiguredMarkdownReplayFamily>, String> {
    let (_, scans) = configured_markdown_corpus_scans(project_root)?;
    let families = scans
        .into_iter()
        .map(|scan| configured_markdown_replay_family(project_root, intent, scan, max_candidates))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    Ok(families)
}

fn configured_markdown_corpus_scans(
    project_root: &Path,
) -> Result<(usize, Vec<ConfiguredMarkdownCorpusScan>), String> {
    let mut visited_config_paths = HashSet::new();
    let mut include_dirs = HashSet::new();
    let mut project_ids = HashSet::new();
    collect_wendao_config_surface(
        project_root,
        Path::new("wendao.toml"),
        &mut visited_config_paths,
        &mut include_dirs,
        &mut project_ids,
    )?;

    let mut sorted_include_dirs = include_dirs.into_iter().collect::<Vec<_>>();
    sorted_include_dirs.sort();
    let scans = sorted_include_dirs
        .into_iter()
        .map(|include_dir| configured_markdown_corpus_scan(project_root, include_dir))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    Ok((project_ids.len(), scans))
}

fn unique_configured_heading_counts(
    scans: &[ConfiguredMarkdownCorpusScan],
) -> HashMap<String, usize> {
    scans.iter().flat_map(|scan| scan.files.iter()).fold(
        HashMap::<String, usize>::new(),
        |mut counts, file| {
            counts
                .entry(file.relative_path.clone())
                .or_insert(file.heading_count);
            counts
        },
    )
}

fn collect_wendao_config_surface(
    project_root: &Path,
    relative_config_path: &Path,
    visited_config_paths: &mut HashSet<PathBuf>,
    include_dirs: &mut HashSet<String>,
    project_ids: &mut HashSet<String>,
) -> Result<(), String> {
    let config_path = project_root.join(relative_config_path);
    if !visited_config_paths.insert(config_path.clone()) {
        return Ok(());
    }

    let config_text = fs::read_to_string(&config_path).map_err(|error| {
        let path = config_path.display();
        format!("read Wendao SearchStrategyFlow config {path}: {error}")
    })?;
    let config = config_text.parse::<toml::Value>().map_err(|error| {
        let path = config_path.display();
        format!("parse Wendao SearchStrategyFlow config {path}: {error}")
    })?;

    if let Some(link_graph) = config.get("link_graph").and_then(toml::Value::as_table) {
        if let Some(dirs) = link_graph
            .get("include_dirs")
            .and_then(toml::Value::as_array)
        {
            include_dirs.extend(
                dirs.iter()
                    .filter_map(toml::Value::as_str)
                    .map(str::to_owned),
            );
        }
        if let Some(projects) = link_graph.get("projects").and_then(toml::Value::as_table) {
            project_ids.extend(projects.keys().cloned());
        }
    }

    if let Some(imports) = config.get("imports").and_then(toml::Value::as_array) {
        imports
            .iter()
            .filter_map(toml::Value::as_str)
            .try_for_each(|import_path| {
                collect_wendao_config_surface(
                    project_root,
                    Path::new(import_path),
                    visited_config_paths,
                    include_dirs,
                    project_ids,
                )
            })?;
    }

    Ok(())
}

fn configured_markdown_corpus_scan(
    project_root: &Path,
    include_dir: String,
) -> Result<Option<ConfiguredMarkdownCorpusScan>, String> {
    let include_root = project_root.join(&include_dir);
    if !include_root.is_dir() {
        return Ok(None);
    }

    let files = configured_markdown_files(include_root.as_path())
        .into_iter()
        .map(|path| configured_markdown_corpus_file(project_root, path.as_path()))
        .collect::<Result<Vec<_>, _>>()?;
    let heading_count = files.iter().map(|file| file.heading_count).sum();

    Ok(Some(ConfiguredMarkdownCorpusScan {
        row: SearchStrategyFlowConfiguredMarkdownCorpusRow {
            include_dir,
            markdown_file_count: files.len(),
            heading_count,
        },
        files,
    }))
}

fn configured_markdown_replay_family(
    project_root: &Path,
    intent: &str,
    scan: ConfiguredMarkdownCorpusScan,
    max_candidates: Option<usize>,
) -> Result<Option<SearchStrategyFlowConfiguredMarkdownReplayFamily>, String> {
    let include_root = project_root.join(&scan.row.include_dir);
    let mut candidates = match max_candidates {
        Some(max_candidates) => discover_search_strategy_flow_candidate_inputs_with_limit(
            intent,
            include_root.as_path(),
            max_candidates,
        )?,
        None => discover_search_strategy_flow_candidate_inputs(intent, include_root.as_path())?,
    };
    if candidates.is_empty() {
        return Ok(None);
    }

    prefix_candidate_paths(scan.row.include_dir.as_str(), &mut candidates);
    Ok(Some(SearchStrategyFlowConfiguredMarkdownReplayFamily {
        include_dir: scan.row.include_dir,
        markdown_file_count: scan.row.markdown_file_count,
        heading_count: scan.row.heading_count,
        batch: search_strategy_flow_candidate_input_batch(
            MARKDOWN_HEADING_CANDIDATE_SOURCE,
            &candidates,
        ),
    }))
}

fn prefix_candidate_paths(include_dir: &str, candidates: &mut [SearchStrategyFlowCandidateInput]) {
    let prefix = include_dir.trim_matches('/');
    if prefix.is_empty() || prefix == "." {
        return;
    }

    for candidate in candidates {
        candidate.relative_path = format!("{prefix}/{}", candidate.relative_path);
    }
}

fn configured_markdown_corpus_file(
    project_root: &Path,
    path: &Path,
) -> Result<ConfiguredMarkdownCorpusFile, String> {
    let text = fs::read_to_string(path).map_err(|error| {
        let path = path.display();
        format!("read configured SearchStrategyFlow Markdown file {path}: {error}")
    })?;
    Ok(ConfiguredMarkdownCorpusFile {
        relative_path: repo_relative_path(project_root, path)?,
        heading_count: heading_sections(&text).len(),
    })
}

fn configured_markdown_files(include_root: &Path) -> Vec<PathBuf> {
    let mut files = WalkDir::new(include_root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !is_ignored_walk_entry(entry))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(DirEntry::into_path)
        .filter(|path| is_markdown_path(path))
        .collect::<Vec<_>>();
    files.sort();
    files
}
