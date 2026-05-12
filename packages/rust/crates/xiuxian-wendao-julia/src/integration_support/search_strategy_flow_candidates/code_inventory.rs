//! Code-Intelligence candidate inventory from git-tracked configured files.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::json;

use super::discovery::{
    markdown_anchor, search_strategy_flow_candidate_input_batch_with_discovery_receipt,
};
use super::types::{
    CODE_INTELLIGENCE_CANDIDATE_SOURCE, SearchStrategyFlowCandidateInput,
    SearchStrategyFlowCandidateInputBatch,
};

const ROOT_WENDAO_CONFIG_PATH: &str = "wendao.toml";
const WENDAO_RUST_SURFACE: &str = "packages/rust/crates/xiuxian-wendao";
const LINK_GRAPH_RUST_SURFACE: &str = "packages/rust/crates/xiuxian-wendao/src/link_graph";
const BENCHMARK_PYTHON_SURFACE: &str = "packages/python/wendao-knowledge-retrieval-benchmark";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchStrategyFlowCodeInventoryAudit {
    pub(crate) configured_include_dirs: Vec<String>,
    pub(crate) primary_markdown_count: usize,
    pub(crate) rust_control_plane_count: usize,
    pub(crate) link_graph_source_count: usize,
    pub(crate) toml_config_count: usize,
    pub(crate) benchmark_python_count: usize,
    pub(crate) total_candidate_count: usize,
}

pub(crate) fn audit_search_strategy_flow_code_intelligence_inventory(
    project_root: &Path,
) -> Result<SearchStrategyFlowCodeInventoryAudit, String> {
    let include_dirs = configured_include_dirs(project_root)?;
    let tracked_paths = git_tracked_paths(project_root)?;
    Ok(code_intelligence_inventory_audit(
        &include_dirs,
        tracked_paths.as_slice(),
    ))
}

pub(crate) fn search_strategy_flow_code_intelligence_inventory_candidate_input_batch(
    project_root: &Path,
) -> Result<SearchStrategyFlowCandidateInputBatch, String> {
    let include_dirs = configured_include_dirs(project_root)?;
    let tracked_paths = git_tracked_paths(project_root)?;
    let candidates =
        code_intelligence_inventory_candidates(&include_dirs, tracked_paths.as_slice());
    let audit = code_intelligence_inventory_audit(&include_dirs, tracked_paths.as_slice());
    let receipt = json!({
        "receiptSource": CODE_INTELLIGENCE_CANDIDATE_SOURCE,
        "candidateInputSource": CODE_INTELLIGENCE_CANDIDATE_SOURCE,
        "candidateInputCount": candidates.len(),
        "transport": "git-ls-files",
        "route": "root-wendao-toml-code-intelligence-inventory",
        "configuredIncludeDirs": audit.configured_include_dirs,
        "rustControlPlaneCount": audit.rust_control_plane_count,
        "linkGraphSourceCount": audit.link_graph_source_count,
        "tomlConfigCount": audit.toml_config_count,
        "benchmarkPythonCount": audit.benchmark_python_count,
        "mergedCandidateCount": candidates.len(),
        "attemptCount": 1,
    });

    Ok(
        search_strategy_flow_candidate_input_batch_with_discovery_receipt(
            CODE_INTELLIGENCE_CANDIDATE_SOURCE,
            &candidates,
            receipt,
        ),
    )
}

fn code_intelligence_inventory_audit(
    include_dirs: &[String],
    tracked_paths: &[String],
) -> SearchStrategyFlowCodeInventoryAudit {
    let candidates = code_intelligence_inventory_candidates(include_dirs, tracked_paths);
    SearchStrategyFlowCodeInventoryAudit {
        configured_include_dirs: include_dirs.to_vec(),
        primary_markdown_count: primary_markdown_candidate_count(include_dirs, tracked_paths),
        rust_control_plane_count: candidates
            .iter()
            .filter(|candidate| has_edge_kind(candidate, "rust-control-plane-source"))
            .count(),
        link_graph_source_count: candidates
            .iter()
            .filter(|candidate| has_edge_kind(candidate, "link-graph-source-focus"))
            .count(),
        toml_config_count: candidates
            .iter()
            .filter(|candidate| has_edge_kind(candidate, "toml-config-boundary"))
            .count(),
        benchmark_python_count: candidates
            .iter()
            .filter(|candidate| has_edge_kind(candidate, "benchmark-python-adapter"))
            .count(),
        total_candidate_count: candidates.len(),
    }
}

fn primary_markdown_candidate_count(include_dirs: &[String], tracked_paths: &[String]) -> usize {
    let include_dirs = include_dirs
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    tracked_paths
        .iter()
        .filter(|path| path.ends_with(".md"))
        .filter(|path| {
            (include_dirs.contains("docs") && path_is_under(path, "docs"))
                || (include_dirs.contains("semantic") && path_is_under(path, "semantic"))
                || (include_dirs.contains(WENDAO_RUST_SURFACE)
                    && path_is_under(path, WENDAO_RUST_SURFACE))
                || (include_dirs.contains(BENCHMARK_PYTHON_SURFACE)
                    && path_is_under(path, BENCHMARK_PYTHON_SURFACE))
        })
        .count()
}

fn code_intelligence_inventory_candidates(
    include_dirs: &[String],
    tracked_paths: &[String],
) -> Vec<SearchStrategyFlowCandidateInput> {
    let include_dirs = include_dirs
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut candidates = Vec::new();
    if include_dirs.contains(WENDAO_RUST_SURFACE) {
        candidates.extend(
            tracked_paths
                .iter()
                .filter(|path| path_is_under(path, WENDAO_RUST_SURFACE))
                .filter(|path| path.ends_with(".rs"))
                .map(|path| code_inventory_candidate(path, "rust-control-plane-source", "rust")),
        );
        candidates.extend(
            tracked_paths
                .iter()
                .filter(|path| path_is_under(path, WENDAO_RUST_SURFACE))
                .filter(|path| path.ends_with(".toml"))
                .map(|path| code_inventory_candidate(path, "toml-config-boundary", "toml")),
        );
    }
    if include_dirs.contains(LINK_GRAPH_RUST_SURFACE) {
        candidates.extend(
            tracked_paths
                .iter()
                .filter(|path| path_is_under(path, LINK_GRAPH_RUST_SURFACE))
                .filter(|path| path.ends_with(".rs"))
                .map(|path| code_inventory_candidate(path, "link-graph-source-focus", "rust")),
        );
    }
    if include_dirs.contains(BENCHMARK_PYTHON_SURFACE) {
        candidates.extend(
            tracked_paths
                .iter()
                .filter(|path| path_is_under(path, BENCHMARK_PYTHON_SURFACE))
                .filter(|path| path.ends_with(".py"))
                .map(|path| code_inventory_candidate(path, "benchmark-python-adapter", "python")),
        );
        candidates.extend(
            tracked_paths
                .iter()
                .filter(|path| path_is_under(path, BENCHMARK_PYTHON_SURFACE))
                .filter(|path| path.ends_with(".toml"))
                .map(|path| code_inventory_candidate(path, "toml-config-boundary", "toml")),
        );
    }
    candidates.sort_by(|left, right| {
        left.heading_anchor
            .cmp(&right.heading_anchor)
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });
    candidates
}

fn code_inventory_candidate(
    relative_path: &str,
    scenario_id: &str,
    language: &str,
) -> SearchStrategyFlowCandidateInput {
    let edge_kinds = vec![
        "code-intelligence".to_owned(),
        "git-tracked-inventory".to_owned(),
        "source-config".to_owned(),
        scenario_id.to_owned(),
        format!("language:{language}"),
    ];
    SearchStrategyFlowCandidateInput {
        relative_path: relative_path.to_owned(),
        heading_anchor: format!("{scenario_id}-{}", markdown_anchor(relative_path)),
        title: format!("Code intelligence {scenario_id}: {relative_path}"),
        line_start: 1,
        line_end: 1,
        context_cost: 8,
        evidence_coverage: 0.84,
        graph_score: if scenario_id == "link-graph-source-focus" {
            0.90
        } else {
            0.82
        },
        authority_score: 0.80,
        structural_score: 0.86,
        uncertainty: 0.12,
        blocked: false,
        edge_kinds,
    }
}

fn configured_include_dirs(project_root: &Path) -> Result<Vec<String>, String> {
    let mut visited_config_paths = BTreeSet::new();
    let mut include_dirs = BTreeSet::new();
    collect_configured_include_dirs(
        project_root,
        Path::new(ROOT_WENDAO_CONFIG_PATH),
        &mut visited_config_paths,
        &mut include_dirs,
    )?;
    Ok(include_dirs.into_iter().collect())
}

fn collect_configured_include_dirs(
    project_root: &Path,
    relative_config_path: &Path,
    visited_config_paths: &mut BTreeSet<PathBuf>,
    include_dirs: &mut BTreeSet<String>,
) -> Result<(), String> {
    let config_path = project_root.join(relative_config_path);
    if !visited_config_paths.insert(config_path.clone()) {
        return Ok(());
    }
    let config_text = fs::read_to_string(&config_path).map_err(|error| {
        let path = config_path.display();
        format!("read Wendao code-intelligence inventory config {path}: {error}")
    })?;
    let config = config_text.parse::<toml::Value>().map_err(|error| {
        let path = config_path.display();
        format!("parse Wendao code-intelligence inventory config {path}: {error}")
    })?;
    if let Some(dirs) = config
        .get("link_graph")
        .and_then(toml::Value::as_table)
        .and_then(|link_graph| link_graph.get("include_dirs"))
        .and_then(toml::Value::as_array)
    {
        include_dirs.extend(
            dirs.iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_owned),
        );
    }
    if let Some(imports) = config.get("imports").and_then(toml::Value::as_array) {
        for import_path in imports.iter().filter_map(toml::Value::as_str) {
            collect_configured_include_dirs(
                project_root,
                Path::new(import_path),
                visited_config_paths,
                include_dirs,
            )?;
        }
    }
    Ok(())
}

fn git_tracked_paths(project_root: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .arg("ls-files")
        .current_dir(project_root)
        .output()
        .map_err(|error| format!("run git ls-files for Code-Intelligence inventory: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-files for Code-Intelligence inventory exited with status {}; stderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .filter(|path| is_inventory_visible_path(path))
        .map(str::to_owned)
        .collect())
}

fn is_inventory_visible_path(path: &str) -> bool {
    !path.starts_with(".cache/")
        && !path.starts_with(".data/")
        && !path.starts_with(".run/")
        && !path.contains("/target/")
        && !path.contains("/node_modules/")
}

fn path_is_under(path: &str, root: &str) -> bool {
    path == root || path.starts_with(&format!("{}/", root.trim_end_matches('/')))
}

fn has_edge_kind(candidate: &SearchStrategyFlowCandidateInput, edge_kind: &str) -> bool {
    candidate.edge_kinds.iter().any(|kind| kind == edge_kind)
}
