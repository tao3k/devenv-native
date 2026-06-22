//! Host-probe execution and report parsing for `WendaoGraph` integration support.

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::scripts::{LINK_GRAPH_HOST_PROBE_JULIA, PAGE_INDEX_HOST_PROBE_JULIA};
use super::{
    LINK_GRAPH_HOST_PROBE_PREFIX, PAGE_INDEX_HOST_PROBE_PREFIX,
    WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_WARM_SAMPLES_ENV,
    WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_FANOUT_ENV, WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_NODES_ENV,
    WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_SEMANTIC_NEIGHBORS_ENV,
    WENDAO_GRAPH_PAGE_INDEX_HOST_FIXTURE_ENV, WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS_ENV,
    WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_WARM_SAMPLES_ENV, WENDAOGRAPH_JULIA_PROJECT_ENV,
    WENDAOGRAPH_PACKAGE_DIR_ENV,
};
use crate::JuliaContractMode;
use crate::integration_support::service_runtime::repo_root;

/// Timing report from one local `WendaoGraph.jl` `PageIndex` host probe.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoGraphPageIndexHostProbeReport {
    /// Number of warm samples measured after the first request.
    pub sample_count: usize,
    /// First host-request call elapsed milliseconds after Julia package load.
    pub first_ms: f64,
    /// Minimum warm-call elapsed milliseconds.
    pub warm_min_ms: f64,
    /// Median warm-call elapsed milliseconds.
    pub warm_median_ms: f64,
    /// P95 warm-call elapsed milliseconds.
    pub warm_p95_ms: f64,
    /// Maximum warm-call elapsed milliseconds.
    pub warm_max_ms: f64,
    /// Reasoning frontier row count returned by the Julia facade.
    pub frontier_rows: usize,
    /// Disclosure trace row count returned by the Julia facade.
    pub trace_rows: usize,
}

/// Timing and action-count report from one local `WendaoGraph.jl` `PageIndex`
/// planner-action host probe.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoGraphPageIndexPlannerActionHostProbeReport {
    /// Base `PageIndex` host-probe timing and row-count report.
    pub base: WendaoGraphPageIndexHostProbeReport,
    /// Planner action row count returned by the Julia facade.
    pub planner_action_rows: usize,
    /// Number of expand actions.
    pub planner_expand_actions: usize,
    /// Number of compare actions.
    pub planner_compare_actions: usize,
    /// Number of jump actions.
    pub planner_jump_actions: usize,
    /// Number of stop actions.
    pub planner_stop_actions: usize,
}

/// Timing report from one local `WendaoGraph.jl` `LinkGraph` host probe.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoGraphLinkGraphHostProbeReport {
    /// Probe input mode selected for the host-process request.
    pub mode: JuliaContractMode,
    /// Number of input graph nodes.
    pub node_count: usize,
    /// Number of input graph edges.
    pub edge_count: usize,
    /// Number of semantic-neighbor or semantic-overlay input rows.
    pub semantic_neighbor_count: usize,
    /// Number of warm samples measured after the first request.
    pub sample_count: usize,
    /// First host-request call elapsed milliseconds after Julia package load.
    pub first_ms: f64,
    /// Minimum warm-call elapsed milliseconds.
    pub warm_min_ms: f64,
    /// Median warm-call elapsed milliseconds.
    pub warm_median_ms: f64,
    /// P95 warm-call elapsed milliseconds.
    pub warm_p95_ms: f64,
    /// Maximum warm-call elapsed milliseconds.
    pub warm_max_ms: f64,
    /// Graph metric row count returned by the Julia facade.
    pub graph_metric_rows: usize,
    /// Topology candidate row count returned by the Julia facade.
    pub topology_candidate_rows: usize,
    /// Semantic overlay row count returned by the Julia facade.
    pub semantic_overlay_rows: usize,
    /// Diffusion score row count returned by the Julia facade.
    pub diffusion_rows: usize,
    /// Link frontier row count returned by the Julia facade.
    pub frontier_rows: usize,
}

/// Timing report plus full structural row counts from one local
/// `WendaoGraph.jl` `LinkGraph` host probe.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoGraphLinkGraphFullStructuralHostProbeReport {
    /// Base `LinkGraph` host-probe timing and core row-count report.
    pub base: WendaoGraphLinkGraphHostProbeReport,
    /// Component row count returned by the Julia facade.
    pub component_rows: usize,
    /// Topology profile row count returned by the Julia facade.
    pub topology_profile_rows: usize,
    /// Topology bottleneck row count returned by the Julia facade.
    pub topology_bottleneck_rows: usize,
    /// Topology community row count returned by the Julia facade.
    pub topology_community_rows: usize,
    /// Topology cover row count returned by the Julia facade.
    pub topology_cover_rows: usize,
    /// Topology core row count returned by the Julia facade.
    pub topology_core_rows: usize,
    /// Topology boundary row count returned by the Julia facade.
    pub topology_boundary_rows: usize,
    /// Topology transition row count returned by the Julia facade.
    pub topology_transition_rows: usize,
    /// Topology gateway row count returned by the Julia facade.
    pub topology_gateway_rows: usize,
    /// Topology community summary row count returned by the Julia facade.
    pub topology_community_summary_rows: usize,
    /// Topology community link row count returned by the Julia facade.
    pub topology_community_link_rows: usize,
    /// Topology community frontier row count returned by the Julia facade.
    pub topology_community_frontier_rows: usize,
}

/// Runs the local `WendaoGraph.jl` `PageIndex` host-request probe in a real
/// Julia process.
///
/// # Errors
///
/// Returns an error when the local `WendaoGraph.jl` project or host fixture
/// cannot be resolved, the Julia process exits unsuccessfully, or the probe
/// output cannot be parsed.
pub fn probe_wendaograph_page_index_host_request(
    warm_sample_count: usize,
) -> Result<WendaoGraphPageIndexHostProbeReport, String> {
    let julia_project = wendaograph_julia_project()?;
    let fixture_dir = wendaograph_page_index_host_fixture_dir()?;
    let stdout = run_wendaograph_page_index_host_probe(
        &julia_project,
        &fixture_dir,
        warm_sample_count,
        false,
        "PageIndex",
    )?;
    parse_page_index_probe_stdout(&stdout)
}

/// Runs the local `WendaoGraph.jl` `PageIndex` host-request probe with an
/// explicit fixture directory.
///
/// # Errors
///
/// Returns an error when the local `WendaoGraph.jl` project or supplied host
/// fixture cannot be resolved, the Julia process exits unsuccessfully, or the
/// probe output cannot be parsed.
pub fn probe_wendaograph_page_index_host_request_with_fixture(
    fixture_dir: impl Into<PathBuf>,
    warm_sample_count: usize,
) -> Result<WendaoGraphPageIndexHostProbeReport, String> {
    let julia_project = wendaograph_julia_project()?;
    let fixture_dir = resolve_existing_path("WendaoGraph PageIndex host fixture", fixture_dir)?;
    let stdout = run_wendaograph_page_index_host_probe(
        &julia_project,
        &fixture_dir,
        warm_sample_count,
        false,
        "PageIndex",
    )?;
    parse_page_index_probe_stdout(&stdout)
}

/// Runs the local `WendaoGraph.jl` `PageIndex` planner-action host probe in a
/// real Julia process.
///
/// # Errors
///
/// Returns an error when the local `WendaoGraph.jl` project or host fixture
/// cannot be resolved, the Julia process exits unsuccessfully, or the probe
/// output cannot be parsed.
pub fn probe_wendaograph_page_index_planner_action_host_request(
    warm_sample_count: usize,
) -> Result<WendaoGraphPageIndexPlannerActionHostProbeReport, String> {
    let julia_project = wendaograph_julia_project()?;
    let fixture_dir = wendaograph_page_index_host_fixture_dir()?;
    let stdout = run_wendaograph_page_index_host_probe(
        &julia_project,
        &fixture_dir,
        warm_sample_count,
        true,
        "PageIndex planner-action",
    )?;
    parse_page_index_planner_action_probe_stdout(&stdout)
}

/// Runs the local `WendaoGraph.jl` `PageIndex` planner-action host probe with
/// an explicit fixture directory.
///
/// # Errors
///
/// Returns an error when the local `WendaoGraph.jl` project or supplied host
/// fixture cannot be resolved, the Julia process exits unsuccessfully, or the
/// probe output cannot be parsed.
pub fn probe_wendaograph_page_index_planner_action_host_request_with_fixture(
    fixture_dir: impl Into<PathBuf>,
    warm_sample_count: usize,
) -> Result<WendaoGraphPageIndexPlannerActionHostProbeReport, String> {
    let julia_project = wendaograph_julia_project()?;
    let fixture_dir = resolve_existing_path("WendaoGraph PageIndex host fixture", fixture_dir)?;
    let stdout = run_wendaograph_page_index_host_probe(
        &julia_project,
        &fixture_dir,
        warm_sample_count,
        true,
        "PageIndex planner-action",
    )?;
    parse_page_index_planner_action_probe_stdout(&stdout)
}

/// Runs the local `WendaoGraph.jl` `LinkGraph` host-request probe in a real
/// Julia process.
///
/// # Errors
///
/// Returns an error when the local `WendaoGraph.jl` project cannot be resolved,
/// the Julia process exits unsuccessfully, or the probe output cannot be parsed.
pub fn probe_wendaograph_link_graph_host_request(
    warm_sample_count: usize,
) -> Result<WendaoGraphLinkGraphHostProbeReport, String> {
    let julia_project = wendaograph_julia_project()?;
    let output = Command::new("julia")
        .arg(format!("--project={}", julia_project.display()))
        .arg("-e")
        .arg(LINK_GRAPH_HOST_PROBE_JULIA)
        .envs(link_graph_synthetic_envs())
        .env(
            WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_WARM_SAMPLES_ENV,
            warm_sample_count.max(1).to_string(),
        )
        .output()
        .map_err(|error| format!("spawn WendaoGraph LinkGraph host probe: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "WendaoGraph LinkGraph host probe exited with status {}; stderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_link_graph_probe_stdout(stdout.as_ref())
}

/// Runs the local `WendaoGraph.jl` `LinkGraph` host-request probe and parses
/// the full structural row-count surface.
///
/// # Errors
///
/// Returns an error when the local `WendaoGraph.jl` project cannot be resolved,
/// the Julia process exits unsuccessfully, or the probe output cannot be parsed.
pub fn probe_wendaograph_link_graph_full_structural_host_request(
    warm_sample_count: usize,
) -> Result<WendaoGraphLinkGraphFullStructuralHostProbeReport, String> {
    let julia_project = wendaograph_julia_project()?;
    let output = Command::new("julia")
        .arg(format!("--project={}", julia_project.display()))
        .arg("-e")
        .arg(LINK_GRAPH_HOST_PROBE_JULIA)
        .envs(link_graph_synthetic_envs())
        .env(
            WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_WARM_SAMPLES_ENV,
            warm_sample_count.max(1).to_string(),
        )
        .output()
        .map_err(|error| {
            format!("spawn WendaoGraph LinkGraph full structural host probe: {error}")
        })?;

    if !output.status.success() {
        return Err(format!(
            "WendaoGraph LinkGraph full structural host probe exited with status {}; stderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_link_graph_full_structural_probe_stdout(stdout.as_ref())
}

pub(crate) fn wendaograph_julia_project() -> Result<PathBuf, String> {
    if let Some(configured) = env::var_os(WENDAOGRAPH_JULIA_PROJECT_ENV) {
        return resolve_existing_path("WendaoGraph Julia project", configured);
    }
    if let Some(configured) = env::var_os(WENDAOGRAPH_PACKAGE_DIR_ENV) {
        return resolve_existing_path("WendaoGraph package dir", configured);
    }

    let candidate = repo_root().join(".data/WendaoGraph.jl");
    if candidate.is_dir() {
        return candidate.canonicalize().map_err(|error| {
            format!(
                "resolve default WendaoGraph package dir `{}`: {error}",
                candidate.display()
            )
        });
    }

    Err(format!(
        "WendaoGraph package dir not found at `{}`; set {WENDAOGRAPH_PACKAGE_DIR_ENV} or {WENDAOGRAPH_JULIA_PROJECT_ENV}",
        candidate.display()
    ))
}

fn wendaograph_page_index_host_fixture_dir() -> Result<PathBuf, String> {
    if let Some(configured) = env::var_os(WENDAO_GRAPH_PAGE_INDEX_HOST_FIXTURE_ENV) {
        return resolve_existing_path("WendaoGraph PageIndex host fixture", configured);
    }
    resolve_existing_path(
        "WendaoGraph PageIndex host fixture",
        repo_root().join(
            "packages/rust/crates/xiuxian-wendao/tests/fixtures/wendaograph_page_index_reasoning_host",
        ),
    )
}

fn run_wendaograph_page_index_host_probe(
    julia_project: &Path,
    fixture_dir: &Path,
    warm_sample_count: usize,
    planner_actions: bool,
    label: &str,
) -> Result<String, String> {
    let mut command = Command::new("julia");
    command
        .arg(format!("--project={}", julia_project.display()))
        .arg("-e")
        .arg(PAGE_INDEX_HOST_PROBE_JULIA)
        .env(WENDAO_GRAPH_PAGE_INDEX_HOST_FIXTURE_ENV, fixture_dir)
        .env(
            WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_WARM_SAMPLES_ENV,
            warm_sample_count.max(1).to_string(),
        );
    if planner_actions {
        command.env(WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS_ENV, "1");
    }

    let output = command
        .output()
        .map_err(|error| format!("spawn WendaoGraph {label} host probe: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "WendaoGraph {label} host probe exited with status {}; stderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn link_graph_synthetic_envs() -> Vec<(&'static str, String)> {
    [
        WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_NODES_ENV,
        WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_FANOUT_ENV,
        WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_SEMANTIC_NEIGHBORS_ENV,
    ]
    .into_iter()
    .filter_map(|key| env::var(key).ok().map(|value| (key, value)))
    .collect()
}

pub(crate) fn resolve_existing_path(
    label: &str,
    configured: impl Into<PathBuf>,
) -> Result<PathBuf, String> {
    let candidate = configured.into();
    let candidate = if candidate.is_absolute() {
        candidate
    } else {
        repo_root().join(candidate)
    };
    candidate
        .canonicalize()
        .map_err(|error| format!("resolve {label} `{}`: {error}", candidate.display()))
}

fn parse_page_index_probe_stdout(
    stdout: &str,
) -> Result<WendaoGraphPageIndexHostProbeReport, String> {
    let line = probe_report_line(stdout, PAGE_INDEX_HOST_PROBE_PREFIX, "PageIndex")?;
    parse_page_index_probe_report_line(line)
}

fn parse_page_index_planner_action_probe_stdout(
    stdout: &str,
) -> Result<WendaoGraphPageIndexPlannerActionHostProbeReport, String> {
    let line = probe_report_line(
        stdout,
        PAGE_INDEX_HOST_PROBE_PREFIX,
        "PageIndex planner action",
    )?;
    parse_page_index_planner_action_probe_report_line(line)
}

fn parse_link_graph_probe_stdout(
    stdout: &str,
) -> Result<WendaoGraphLinkGraphHostProbeReport, String> {
    let line = probe_report_line(stdout, LINK_GRAPH_HOST_PROBE_PREFIX, "LinkGraph")?;
    parse_link_graph_probe_report_line(line)
}

fn parse_link_graph_full_structural_probe_stdout(
    stdout: &str,
) -> Result<WendaoGraphLinkGraphFullStructuralHostProbeReport, String> {
    let line = probe_report_line(
        stdout,
        LINK_GRAPH_HOST_PROBE_PREFIX,
        "LinkGraph full structural",
    )?;
    parse_link_graph_full_structural_probe_report_line(line)
}

fn probe_report_line<'a>(stdout: &'a str, prefix: &str, label: &str) -> Result<&'a str, String> {
    stdout
        .lines()
        .find(|line| line.starts_with(prefix))
        .ok_or_else(|| {
            format!("WendaoGraph {label} host probe did not print `{prefix}`; stdout:\n{stdout}")
        })
}

pub(crate) fn parse_page_index_probe_report_line(
    line: &str,
) -> Result<WendaoGraphPageIndexHostProbeReport, String> {
    let fields = parse_probe_fields(line)?;

    Ok(WendaoGraphPageIndexHostProbeReport {
        sample_count: parse_usize_field(&fields, "sample_count")?,
        first_ms: parse_f64_field(&fields, "first_ms")?,
        warm_min_ms: parse_f64_field(&fields, "warm_min_ms")?,
        warm_median_ms: parse_f64_field(&fields, "warm_median_ms")?,
        warm_p95_ms: parse_f64_field(&fields, "warm_p95_ms")?,
        warm_max_ms: parse_f64_field(&fields, "warm_max_ms")?,
        frontier_rows: parse_usize_field(&fields, "frontier_rows")?,
        trace_rows: parse_usize_field(&fields, "trace_rows")?,
    })
}

pub(crate) fn parse_page_index_planner_action_probe_report_line(
    line: &str,
) -> Result<WendaoGraphPageIndexPlannerActionHostProbeReport, String> {
    let fields = parse_probe_fields(line)?;

    Ok(WendaoGraphPageIndexPlannerActionHostProbeReport {
        base: WendaoGraphPageIndexHostProbeReport {
            sample_count: parse_usize_field(&fields, "sample_count")?,
            first_ms: parse_f64_field(&fields, "first_ms")?,
            warm_min_ms: parse_f64_field(&fields, "warm_min_ms")?,
            warm_median_ms: parse_f64_field(&fields, "warm_median_ms")?,
            warm_p95_ms: parse_f64_field(&fields, "warm_p95_ms")?,
            warm_max_ms: parse_f64_field(&fields, "warm_max_ms")?,
            frontier_rows: parse_usize_field(&fields, "frontier_rows")?,
            trace_rows: parse_usize_field(&fields, "trace_rows")?,
        },
        planner_action_rows: parse_usize_field(&fields, "planner_action_rows")?,
        planner_expand_actions: parse_usize_field(&fields, "planner_expand_actions")?,
        planner_compare_actions: parse_usize_field(&fields, "planner_compare_actions")?,
        planner_jump_actions: parse_usize_field(&fields, "planner_jump_actions")?,
        planner_stop_actions: parse_usize_field(&fields, "planner_stop_actions")?,
    })
}

pub(crate) fn parse_link_graph_probe_report_line(
    line: &str,
) -> Result<WendaoGraphLinkGraphHostProbeReport, String> {
    let fields = parse_probe_fields(line)?;

    Ok(WendaoGraphLinkGraphHostProbeReport {
        mode: parse_string_field_or(&fields, "mode", "semantic-neighbors").into(),
        node_count: parse_usize_field_or(&fields, "node_count", 4)?,
        edge_count: parse_usize_field_or(&fields, "edge_count", 2)?,
        semantic_neighbor_count: parse_usize_field_or(&fields, "semantic_neighbor_count", 1)?,
        sample_count: parse_usize_field(&fields, "sample_count")?,
        first_ms: parse_f64_field(&fields, "first_ms")?,
        warm_min_ms: parse_f64_field(&fields, "warm_min_ms")?,
        warm_median_ms: parse_f64_field(&fields, "warm_median_ms")?,
        warm_p95_ms: parse_f64_field(&fields, "warm_p95_ms")?,
        warm_max_ms: parse_f64_field(&fields, "warm_max_ms")?,
        graph_metric_rows: parse_usize_field(&fields, "graph_metric_rows")?,
        topology_candidate_rows: parse_usize_field(&fields, "topology_candidate_rows")?,
        semantic_overlay_rows: parse_usize_field(&fields, "semantic_overlay_rows")?,
        diffusion_rows: parse_usize_field(&fields, "diffusion_rows")?,
        frontier_rows: parse_usize_field(&fields, "frontier_rows")?,
    })
}

pub(crate) fn parse_link_graph_full_structural_probe_report_line(
    line: &str,
) -> Result<WendaoGraphLinkGraphFullStructuralHostProbeReport, String> {
    let fields = parse_probe_fields(line)?;

    Ok(WendaoGraphLinkGraphFullStructuralHostProbeReport {
        base: WendaoGraphLinkGraphHostProbeReport {
            mode: parse_string_field_or(&fields, "mode", "semantic-neighbors").into(),
            node_count: parse_usize_field_or(&fields, "node_count", 4)?,
            edge_count: parse_usize_field_or(&fields, "edge_count", 2)?,
            semantic_neighbor_count: parse_usize_field_or(&fields, "semantic_neighbor_count", 1)?,
            sample_count: parse_usize_field(&fields, "sample_count")?,
            first_ms: parse_f64_field(&fields, "first_ms")?,
            warm_min_ms: parse_f64_field(&fields, "warm_min_ms")?,
            warm_median_ms: parse_f64_field(&fields, "warm_median_ms")?,
            warm_p95_ms: parse_f64_field(&fields, "warm_p95_ms")?,
            warm_max_ms: parse_f64_field(&fields, "warm_max_ms")?,
            graph_metric_rows: parse_usize_field(&fields, "graph_metric_rows")?,
            topology_candidate_rows: parse_usize_field(&fields, "topology_candidate_rows")?,
            semantic_overlay_rows: parse_usize_field(&fields, "semantic_overlay_rows")?,
            diffusion_rows: parse_usize_field(&fields, "diffusion_rows")?,
            frontier_rows: parse_usize_field(&fields, "frontier_rows")?,
        },
        component_rows: parse_usize_field(&fields, "component_rows")?,
        topology_profile_rows: parse_usize_field(&fields, "topology_profile_rows")?,
        topology_bottleneck_rows: parse_usize_field(&fields, "topology_bottleneck_rows")?,
        topology_community_rows: parse_usize_field(&fields, "topology_community_rows")?,
        topology_cover_rows: parse_usize_field(&fields, "topology_cover_rows")?,
        topology_core_rows: parse_usize_field(&fields, "topology_core_rows")?,
        topology_boundary_rows: parse_usize_field(&fields, "topology_boundary_rows")?,
        topology_transition_rows: parse_usize_field(&fields, "topology_transition_rows")?,
        topology_gateway_rows: parse_usize_field(&fields, "topology_gateway_rows")?,
        topology_community_summary_rows: parse_usize_field(
            &fields,
            "topology_community_summary_rows",
        )?,
        topology_community_link_rows: parse_usize_field(&fields, "topology_community_link_rows")?,
        topology_community_frontier_rows: parse_usize_field(
            &fields,
            "topology_community_frontier_rows",
        )?,
    })
}

fn parse_probe_fields(line: &str) -> Result<HashMap<&str, &str>, String> {
    let mut fields = HashMap::new();
    for token in line.split_whitespace().skip(1) {
        let (key, value) = token
            .split_once('=')
            .ok_or_else(|| format!("invalid probe token `{token}`"))?;
        fields.insert(key, value);
    }
    Ok(fields)
}

fn parse_usize_field(fields: &HashMap<&str, &str>, key: &str) -> Result<usize, String> {
    fields
        .get(key)
        .ok_or_else(|| format!("missing probe field `{key}`"))?
        .parse()
        .map_err(|error| format!("parse probe field `{key}` as usize: {error}"))
}

fn parse_usize_field_or(
    fields: &HashMap<&str, &str>,
    key: &str,
    default_value: usize,
) -> Result<usize, String> {
    fields.get(key).map_or(Ok(default_value), |value| {
        value
            .parse()
            .map_err(|error| format!("parse probe field `{key}` as usize: {error}"))
    })
}

fn parse_string_field_or<'a>(
    fields: &'a HashMap<&str, &str>,
    key: &str,
    default_value: &'a str,
) -> &'a str {
    fields.get(key).copied().unwrap_or(default_value)
}

fn parse_f64_field(fields: &HashMap<&str, &str>, key: &str) -> Result<f64, String> {
    fields
        .get(key)
        .ok_or_else(|| format!("missing probe field `{key}`"))?
        .parse()
        .map_err(|error| format!("parse probe field `{key}` as f64: {error}"))
}
