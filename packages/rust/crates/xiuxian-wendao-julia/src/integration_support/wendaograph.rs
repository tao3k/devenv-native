//! Host-process probes for local `WendaoGraph.jl` contracts.

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::service_runtime::repo_root;

const WENDAOGRAPH_PACKAGE_DIR_ENV: &str = "WENDAOGRAPH_PACKAGE_DIR";
const WENDAOGRAPH_JULIA_PROJECT_ENV: &str = "WENDAOGRAPH_JULIA_PROJECT";
const WENDAO_GRAPH_PAGE_INDEX_HOST_FIXTURE_ENV: &str = "WENDAO_GRAPH_PAGE_INDEX_HOST_FIXTURE";
const WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS_ENV: &str =
    "WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS";
const WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_WARM_SAMPLES_ENV: &str =
    "WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_WARM_SAMPLES";
const WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_WARM_SAMPLES_ENV: &str =
    "WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_WARM_SAMPLES";
const WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_NODES_ENV: &str = "WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_NODES";
const WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_FANOUT_ENV: &str =
    "WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_FANOUT";
const WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_SEMANTIC_NEIGHBORS_ENV: &str =
    "WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_SEMANTIC_NEIGHBORS";
const PAGE_INDEX_HOST_PROBE_PREFIX: &str = "wendaograph_page_index_host_probe";
const LINK_GRAPH_HOST_PROBE_PREFIX: &str = "wendaograph_link_graph_host_probe";

const PAGE_INDEX_HOST_PROBE_JULIA: &str = r#"
using WendaoGraph

function tsv_rows(path)
    lines = split(read(path, String), '\n')
    isempty(lines) && return String[], Vector{Vector{String}}()
    header = split(first(lines), '\t'; keepempty = true)
    rows = Vector{Vector{String}}()
    for line in lines[2:end]
        isempty(strip(line)) && continue
        push!(rows, split(line, '\t'; keepempty = true))
    end

    header, rows
end

function require_header(header, expected, subject)
    String.(header) == collect(String.(expected)) ||
        error("$subject header mismatch")
end

function page_index_nodes_from_fixture(fixture_dir)
    header, rows = tsv_rows(joinpath(fixture_dir, "page_index_nodes.tsv"))
    require_header(header, page_index_node_columns(), "page_index_nodes")
    page_index_node_columntable([
        (
            node_id = row[1],
            page_id = row[2],
            parent_id = row[3],
            depth = parse(Int, row[4]),
            rank = parse(Int, row[5]),
            title = row[6],
            summary = row[7],
            line_start = parse(Int, row[8]),
            line_end = parse(Int, row[9]),
            token_count = parse(Int, row[10]),
        ) for row in rows
    ])
end

function page_index_edges_from_fixture(fixture_dir)
    header, rows = tsv_rows(joinpath(fixture_dir, "page_index_edges.tsv"))
    require_header(header, page_index_edge_columns(), "page_index_edges")
    page_index_edge_columntable([
        (
            source_id = row[1],
            target_id = row[2],
            edge_kind = row[3],
            weight = parse(Float64, row[4]),
        ) for row in rows
    ])
end

function page_index_seeds_from_fixture(fixture_dir)
    header, rows = tsv_rows(joinpath(fixture_dir, "page_index_seeds.tsv"))
    require_header(header, page_index_seed_columns(), "page_index_seeds")
    page_index_seed_columntable([
        (node_id = row[1], weight = parse(Float64, row[2]), seed_kind = row[3]) for
        row in rows
    ])
end

function timed_request(request)
    started = time_ns()
    result = page_index_reasoning_from_request(
        request;
        max_depth = 1,
        fanout = 1,
        tree_id = "host-probe",
    )
    elapsed_ms = (time_ns() - started) / 1_000_000
    elapsed_ms, result
end

function percentile(sorted_samples, ratio)
    index = clamp(ceil(Int, length(sorted_samples) * ratio), 1, length(sorted_samples))
    sorted_samples[index]
end

function truthy_env(name)
    lowercase(get(ENV, name, "0")) in ("1", "true", "yes", "on")
end

function planner_action_counts(request, result)
    actions = page_index_planner_action_table(
        result.reasoning_frontier;
        node_ids = request.page_index_nodes.node_id,
        jump_targets = ["docs/beta#beta"],
        stop_threshold = 1.0,
    )
    validate_page_index_planner_action_table(
        actions;
        frontier = result.reasoning_frontier,
        node_ids = request.page_index_nodes.node_id,
    )
    kind_counts = Dict("expand" => 0, "compare" => 0, "jump" => 0, "stop" => 0)
    for action_kind in actions.action_kind
        kind = String(action_kind)
        kind_counts[kind] = get(kind_counts, kind, 0) + 1
    end

    (
        rows = length(actions.action_id),
        expand = kind_counts["expand"],
        compare = kind_counts["compare"],
        jump = kind_counts["jump"],
        stop = kind_counts["stop"],
    )
end

function render_probe_report()
    fixture_dir = ENV["WENDAO_GRAPH_PAGE_INDEX_HOST_FIXTURE"]
    sample_count = max(
        parse(Int, get(ENV, "WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_WARM_SAMPLES", "3")),
        1,
    )
    request = (
        page_index_nodes = page_index_nodes_from_fixture(fixture_dir),
        page_index_edges = page_index_edges_from_fixture(fixture_dir),
        page_index_seeds = page_index_seeds_from_fixture(fixture_dir),
    )

    first_ms, first_result = timed_request(request)
    validate_page_index_reasoning_tables(first_result)
    frontier_rows = length(first_result.reasoning_frontier.node_id)
    trace_rows = length(first_result.disclosure_trace.step_id)
    action_counts = truthy_env("WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS") ?
                    planner_action_counts(request, first_result) :
                    (rows = 0, expand = 0, compare = 0, jump = 0, stop = 0)

    samples = Float64[]
    for _ in 1:sample_count
        elapsed_ms, result = timed_request(request)
        validate_page_index_reasoning_tables(result)
        length(result.reasoning_frontier.node_id) == frontier_rows ||
            error("frontier row count changed")
        length(result.disclosure_trace.step_id) == trace_rows ||
            error("trace row count changed")
        if truthy_env("WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS")
            planner_action_counts(request, result) == action_counts ||
                error("planner action counts changed")
        end
        push!(samples, elapsed_ms)
    end
    sorted_samples = sort(samples)

    println(
        "wendaograph_page_index_host_probe " *
        "sample_count=$(sample_count) " *
        "first_ms=$(round(first_ms; digits = 3)) " *
        "warm_min_ms=$(round(sorted_samples[begin]; digits = 3)) " *
        "warm_median_ms=$(round(percentile(sorted_samples, 0.5); digits = 3)) " *
        "warm_p95_ms=$(round(percentile(sorted_samples, 0.95); digits = 3)) " *
        "warm_max_ms=$(round(last(sorted_samples); digits = 3)) " *
        "frontier_rows=$(frontier_rows) " *
        "trace_rows=$(trace_rows) " *
        "planner_action_rows=$(action_counts.rows) " *
        "planner_expand_actions=$(action_counts.expand) " *
        "planner_compare_actions=$(action_counts.compare) " *
        "planner_jump_actions=$(action_counts.jump) " *
        "planner_stop_actions=$(action_counts.stop)",
    )
end

render_probe_report()
"#;

const LINK_GRAPH_HOST_PROBE_JULIA: &str = r#"
using WendaoGraph

function request_roots(request)
    hasproperty(request, :seeds) && length(request.seeds.node_id) > 0 &&
        return [String(request.seeds.node_id[1])]
    [String(request.nodes.id[1])]
end

function timed_request(request)
    started = time_ns()
    result = link_graph_evidence_from_request(
        request;
        component_kinds = :weak,
        hnsw_bidirectional = true,
        max_depth = 1,
        fanout = 2,
        roots = request_roots(request),
        tree_id = "host-probe",
    )
    elapsed_ms = (time_ns() - started) / 1_000_000
    elapsed_ms, result
end

function percentile(sorted_samples, ratio)
    index = clamp(ceil(Int, length(sorted_samples) * ratio), 1, length(sorted_samples))
    sorted_samples[index]
end

function base_link_graph_request()
    (
        nodes = (id = ["alpha", "beta", "gamma", "delta"],),
        edges = (source_id = ["alpha", "beta"], target_id = ["beta", "gamma"]),
        seeds = diffusion_seed_columntable([diffusion_seed_row("alpha")]),
    )
end

function semantic_neighbor_request()
    base = base_link_graph_request()
    merge(base, (semantic_neighbors = semantic_neighbor_columntable([(
        query_id = "alpha",
        neighbor_id = "delta",
        query_index = 1,
        neighbor_index = 4,
        rank = 1,
        distance = 0.0,
    ),]),))
end

function semantic_overlay_request()
    base = base_link_graph_request()
    merge(base, (semantic_overlay = semantic_overlay_columntable([
        (
            source_id = "alpha",
            target_id = "delta",
            source_index = 1,
            target_index = 4,
            rank = 1,
            distance = 0.0,
            weight = 1.0,
            edge_kind = "semantic",
        ),
        (
            source_id = "delta",
            target_id = "alpha",
            source_index = 4,
            target_index = 1,
            rank = 1,
            distance = 0.0,
            weight = 1.0,
            edge_kind = "semantic",
        ),
    ]),))
end

function env_int(name, default_value)
    max(parse(Int, get(ENV, name, string(default_value))), 1)
end

function synthetic_large_request()
    node_count = max(env_int("WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_NODES", 256), 4)
    fanout = min(
        max(env_int("WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_FANOUT", 4), 1),
        node_count - 1,
    )
    semantic_neighbor_count = min(
        max(env_int("WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_SEMANTIC_NEIGHBORS", node_count), 1),
        node_count,
    )
    ids = ["node_$(index)" for index in 1:node_count]
    sources = String[]
    targets = String[]
    for source_index in 1:node_count
        for offset in 1:fanout
            push!(sources, ids[source_index])
            push!(targets, ids[((source_index + offset - 1) % node_count) + 1])
        end
    end
    semantic_neighbors = [
        (
            query_id = ids[1],
            neighbor_id = ids[index],
            query_index = 1,
            neighbor_index = index,
            rank = index,
            distance = Float64(index - 1) / max(semantic_neighbor_count, 1),
        ) for index in 1:semantic_neighbor_count
    ]

    (
        nodes = (id = ids,),
        edges = (source_id = sources, target_id = targets),
        seeds = diffusion_seed_columntable([diffusion_seed_row(ids[1])]),
        semantic_neighbors = semantic_neighbor_columntable(semantic_neighbors),
    )
end

function request_node_count(request)
    length(request.nodes.id)
end

function request_edge_count(request)
    length(request.edges.source_id)
end

function request_semantic_neighbor_count(request)
    hasproperty(request, :semantic_neighbors) && return length(request.semantic_neighbors.query_id)
    hasproperty(request, :semantic_overlay) && return length(request.semantic_overlay.source_id)
    0
end

function link_graph_probe_request(mode)
    mode == "semantic-neighbors" && return semantic_neighbor_request()
    mode == "semantic-overlay" && return semantic_overlay_request()
    mode == "synthetic-large" && return synthetic_large_request()
    error("unsupported WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_MODE=$(mode)")
end

function render_probe_report()
    sample_count = max(
        parse(Int, get(ENV, "WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_WARM_SAMPLES", "3")),
        1,
    )
    mode = get(ENV, "WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_MODE", "semantic-neighbors")
    request = link_graph_probe_request(mode)
    node_count = request_node_count(request)
    edge_count = request_edge_count(request)
    semantic_neighbor_count = request_semantic_neighbor_count(request)

    first_ms, first_result = timed_request(request)
    first_counts = validate_link_graph_evidence_tables(first_result)

    samples = Float64[]
    for _ in 1:sample_count
        elapsed_ms, result = timed_request(request)
        validate_link_graph_evidence_tables(result) == first_counts ||
            error("LinkGraph evidence row counts changed")
        push!(samples, elapsed_ms)
    end
    sorted_samples = sort(samples)

    println(
        "wendaograph_link_graph_host_probe " *
        "mode=$(mode) " *
        "node_count=$(node_count) " *
        "edge_count=$(edge_count) " *
        "semantic_neighbor_count=$(semantic_neighbor_count) " *
        "sample_count=$(sample_count) " *
        "first_ms=$(round(first_ms; digits = 3)) " *
        "warm_min_ms=$(round(sorted_samples[begin]; digits = 3)) " *
        "warm_median_ms=$(round(percentile(sorted_samples, 0.5); digits = 3)) " *
        "warm_p95_ms=$(round(percentile(sorted_samples, 0.95); digits = 3)) " *
        "warm_max_ms=$(round(last(sorted_samples); digits = 3)) " *
        "graph_metric_rows=$(first_counts.graph_metrics) " *
        "component_rows=$(first_counts.components) " *
        "topology_profile_rows=$(first_counts.topology_profile) " *
        "topology_candidate_rows=$(first_counts.topology_candidates) " *
        "topology_bottleneck_rows=$(first_counts.topology_bottlenecks) " *
        "topology_community_rows=$(first_counts.topology_communities) " *
        "topology_cover_rows=$(first_counts.topology_cover) " *
        "topology_core_rows=$(first_counts.topology_core) " *
        "topology_boundary_rows=$(first_counts.topology_boundary) " *
        "topology_transition_rows=$(first_counts.topology_transitions) " *
        "topology_gateway_rows=$(first_counts.topology_gateways) " *
        "topology_community_summary_rows=$(first_counts.topology_community_summaries) " *
        "topology_community_link_rows=$(first_counts.topology_community_links) " *
        "topology_community_frontier_rows=$(first_counts.topology_community_frontier) " *
        "semantic_overlay_rows=$(first_counts.semantic_overlay) " *
        "diffusion_rows=$(first_counts.diffusion_scores) " *
        "frontier_rows=$(first_counts.link_frontier)",
    )
end

render_probe_report()
"#;

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
    pub mode: String,
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

fn wendaograph_julia_project() -> Result<PathBuf, String> {
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

fn resolve_existing_path(label: &str, configured: impl Into<PathBuf>) -> Result<PathBuf, String> {
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

fn parse_page_index_probe_report_line(
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

fn parse_page_index_planner_action_probe_report_line(
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

fn parse_link_graph_probe_report_line(
    line: &str,
) -> Result<WendaoGraphLinkGraphHostProbeReport, String> {
    let fields = parse_probe_fields(line)?;

    Ok(WendaoGraphLinkGraphHostProbeReport {
        mode: parse_string_field_or(&fields, "mode", "semantic-neighbors").to_owned(),
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

fn parse_link_graph_full_structural_probe_report_line(
    line: &str,
) -> Result<WendaoGraphLinkGraphFullStructuralHostProbeReport, String> {
    let fields = parse_probe_fields(line)?;

    Ok(WendaoGraphLinkGraphFullStructuralHostProbeReport {
        base: WendaoGraphLinkGraphHostProbeReport {
            mode: parse_string_field_or(&fields, "mode", "semantic-neighbors").to_owned(),
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

#[cfg(test)]
#[path = "../../tests/unit/integration_support/wendaograph.rs"]
mod tests;
