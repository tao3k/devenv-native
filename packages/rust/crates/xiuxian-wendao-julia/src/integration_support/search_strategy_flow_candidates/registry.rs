//! Registry-authority candidate batches from the root `wendao.toml` surface.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

use super::discovery::{
    line_context_cost, markdown_anchor, repo_relative_path,
    search_strategy_flow_candidate_input_batch_with_discovery_receipt,
};
use super::structured_index::REGISTRY_METADATA_CANDIDATE_SOURCE;
use super::types::{SearchStrategyFlowCandidateInput, SearchStrategyFlowCandidateInputBatch};

const ROOT_WENDAO_CONFIG_PATH: &str = "wendao.toml";
const ROOT_WENDAO_CONFIG_SURFACE: &str = "root-wendao.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchStrategyFlowRegistryAuthorityAudit {
    pub(crate) config_surface: String,
    pub(crate) configured_project_count: usize,
    pub(crate) local_project_count: usize,
    pub(crate) remote_project_count: usize,
    pub(crate) visited_config_count: usize,
    pub(crate) rows: Vec<SearchStrategyFlowRegistryAuthorityProject>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchStrategyFlowRegistryAuthorityProject {
    pub(crate) project_id: String,
    pub(crate) config_path: String,
    pub(crate) line_start: usize,
    pub(crate) line_end: usize,
    pub(crate) root: Option<String>,
    pub(crate) url: Option<String>,
    pub(crate) refresh: Option<String>,
    pub(crate) dirs: Vec<String>,
    pub(crate) plugins: Vec<String>,
}

pub(crate) fn audit_search_strategy_flow_registry_authority(
    project_root: &Path,
) -> Result<SearchStrategyFlowRegistryAuthorityAudit, String> {
    let mut visited_config_paths = HashSet::new();
    let mut projects = BTreeMap::new();
    collect_registry_authority_surface(
        project_root,
        Path::new(ROOT_WENDAO_CONFIG_PATH),
        &mut visited_config_paths,
        &mut projects,
    )?;

    let rows = projects.into_values().collect::<Vec<_>>();
    let local_project_count = rows.iter().filter(|project| project.root.is_some()).count();
    let remote_project_count = rows.iter().filter(|project| project.url.is_some()).count();

    Ok(SearchStrategyFlowRegistryAuthorityAudit {
        config_surface: ROOT_WENDAO_CONFIG_SURFACE.to_owned(),
        configured_project_count: rows.len(),
        local_project_count,
        remote_project_count,
        visited_config_count: visited_config_paths.len(),
        rows,
    })
}

pub(crate) fn search_strategy_flow_registry_authority_candidate_input_batch(
    project_root: &Path,
) -> Result<SearchStrategyFlowCandidateInputBatch, String> {
    let audit = audit_search_strategy_flow_registry_authority(project_root)?;
    let candidates = audit
        .rows
        .iter()
        .map(registry_authority_candidate_input)
        .collect::<Vec<_>>();
    let receipt = json!({
        "receiptSource": REGISTRY_METADATA_CANDIDATE_SOURCE,
        "candidateInputSource": REGISTRY_METADATA_CANDIDATE_SOURCE,
        "candidateInputCount": candidates.len(),
        "transport": "rust-config-scan",
        "route": "root-wendao-toml-registry-authority",
        "configSurface": audit.config_surface,
        "configuredProjectCount": audit.configured_project_count,
        "localProjectCount": audit.local_project_count,
        "remoteProjectCount": audit.remote_project_count,
        "attemptCount": audit.visited_config_count,
        "mergedCandidateCount": candidates.len(),
    });
    Ok(
        search_strategy_flow_candidate_input_batch_with_discovery_receipt(
            REGISTRY_METADATA_CANDIDATE_SOURCE,
            &candidates,
            receipt,
        ),
    )
}

fn collect_registry_authority_surface(
    project_root: &Path,
    relative_config_path: &Path,
    visited_config_paths: &mut HashSet<PathBuf>,
    projects: &mut BTreeMap<String, SearchStrategyFlowRegistryAuthorityProject>,
) -> Result<(), String> {
    let config_path = project_root.join(relative_config_path);
    if !visited_config_paths.insert(config_path.clone()) {
        return Ok(());
    }

    let config_text = fs::read_to_string(&config_path).map_err(|error| {
        let path = config_path.display();
        format!("read Wendao registry-authority config {path}: {error}")
    })?;
    let config = config_text.parse::<toml::Value>().map_err(|error| {
        let path = config_path.display();
        format!("parse Wendao registry-authority config {path}: {error}")
    })?;
    let config_relative_path = repo_relative_path(project_root, config_path.as_path())?;

    if let Some(project_tables) = config
        .get("link_graph")
        .and_then(toml::Value::as_table)
        .and_then(|link_graph| link_graph.get("projects"))
        .and_then(toml::Value::as_table)
    {
        for (project_id, project) in project_tables {
            projects.entry(project_id.clone()).or_insert_with(|| {
                registry_authority_project(project_id, project, &config_text, &config_relative_path)
            });
        }
    }

    if let Some(imports) = config.get("imports").and_then(toml::Value::as_array) {
        for import_path in imports.iter().filter_map(toml::Value::as_str) {
            collect_registry_authority_surface(
                project_root,
                Path::new(import_path),
                visited_config_paths,
                projects,
            )?;
        }
    }

    Ok(())
}

fn registry_authority_project(
    project_id: &str,
    project: &toml::Value,
    config_text: &str,
    config_relative_path: &str,
) -> SearchStrategyFlowRegistryAuthorityProject {
    let table_line = project_table_line(config_text, project_id).unwrap_or(1);
    SearchStrategyFlowRegistryAuthorityProject {
        project_id: project_id.to_owned(),
        config_path: config_relative_path.to_owned(),
        line_start: table_line,
        line_end: table_line,
        root: project
            .get("root")
            .and_then(toml::Value::as_str)
            .map(str::to_owned),
        url: project
            .get("url")
            .and_then(toml::Value::as_str)
            .map(str::to_owned),
        refresh: project
            .get("refresh")
            .and_then(toml::Value::as_str)
            .map(str::to_owned),
        dirs: string_array(project, "dirs"),
        plugins: string_array(project, "plugins"),
    }
}

fn registry_authority_candidate_input(
    project: &SearchStrategyFlowRegistryAuthorityProject,
) -> SearchStrategyFlowCandidateInput {
    let mut edge_kinds = vec![
        "authority".to_owned(),
        "registry-authority".to_owned(),
        "source-authority".to_owned(),
        "package-owner".to_owned(),
        "project-metadata".to_owned(),
        "wendao-config".to_owned(),
        "rust-discovered".to_owned(),
    ];
    if project.url.is_some() {
        edge_kinds.push("remote-resource".to_owned());
    }
    if project.root.is_some() {
        edge_kinds.push("local-resource".to_owned());
    }
    edge_kinds.extend(
        project
            .plugins
            .iter()
            .map(|plugin| format!("plugin:{plugin}")),
    );
    edge_kinds.sort();
    edge_kinds.dedup();

    SearchStrategyFlowCandidateInput {
        relative_path: project.config_path.clone(),
        heading_anchor: format!(
            "registry-authority-source-authority-package-owner-{}",
            markdown_anchor(&project.project_id)
        ),
        title: format!("Registry authority: {}", project.project_id),
        line_start: project.line_start,
        line_end: project.line_end,
        context_cost: line_context_cost(project.line_start, project.line_end)
            + project.dirs.len().saturating_mul(2)
            + project.plugins.len().saturating_mul(2),
        evidence_coverage: 0.92,
        graph_score: 0.68,
        authority_score: 0.96,
        structural_score: 0.82,
        uncertainty: 0.05,
        blocked: false,
        edge_kinds,
    }
}

fn project_table_line(config_text: &str, project_id: &str) -> Option<usize> {
    config_text
        .lines()
        .enumerate()
        .find_map(|(index, line)| project_table_line_matches(line, project_id).then_some(index + 1))
}

fn project_table_line_matches(line: &str, project_id: &str) -> bool {
    let line = line.trim();
    let Some(body) = line
        .strip_prefix("[link_graph.projects.")
        .and_then(|line| line.strip_suffix(']'))
    else {
        return false;
    };
    body.trim_matches('"') == project_id
}

fn string_array(project: &toml::Value, key: &str) -> Vec<String> {
    project
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}
