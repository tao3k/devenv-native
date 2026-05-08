//! Host-process probe for local `WendaoGraph.jl` GNN reasoning contracts.

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Command;

use super::service_runtime::repo_root;

const WENDAOGRAPH_PACKAGE_DIR_ENV: &str = "WENDAOGRAPH_PACKAGE_DIR";
const WENDAOGRAPH_JULIA_PROJECT_ENV: &str = "WENDAOGRAPH_JULIA_PROJECT";
const WENDAO_GRAPH_GNN_HOST_PROBE_WARM_SAMPLES_ENV: &str =
    "WENDAO_GRAPH_GNN_HOST_PROBE_WARM_SAMPLES";
const GNN_HOST_PROBE_PREFIX: &str = "wendaograph_gnn_host_probe";

const GNN_HOST_PROBE_JULIA: &str = r#"
using WendaoGraph

const METAL_IMPORT_ERROR = Ref("")

try
    @static if Sys.isapple()
        @eval import Metal
    end
catch error
    METAL_IMPORT_ERROR[] = sprint(showerror, error)
end

function probe_payload()
    nodes = (id = ["alpha", "beta", "gamma", "delta"],)
    edges = (
        source_id = ["alpha", "alpha", "beta", "gamma"],
        target_id = ["beta", "gamma", "delta", "delta"],
    )
    snapshot = build_graph_snapshot(nodes, edges)
    embeddings = [
        Float32[1.0, 0.0],
        Float32[0.8, 0.2],
        Float32[0.2, 0.9],
        Float32[0.4, 0.7],
    ]

    features = gnn_node_features(snapshot; embeddings = embeddings)
    graph = gnn_graph(snapshot; node_features = features)
    model = build_gcn_frontier_model(size(features, 1); hidden_dim = 4, seed = 7)
    scores = gnn_node_scores(model, graph; device = :cpu)
    length(scores) == length(nodes.id) || error("CPU score count mismatch")
    all(isfinite, scores) || error("CPU scores must be finite")

    frontier = reasoning_frontier_rows(
        snapshot,
        [0.1, 0.9, 0.4, 0.7];
        roots = ["alpha"],
        max_depth = 2,
        fanout = 1,
        direction = :out,
        tree_id = "gnn-host",
    )
    length(frontier) == 3 || error("unexpected GNN frontier row count")

    status = gnn_backend_status()
    metal_functional = false
    metal_score_count = 0

    if status.metal_loaded && isdefined(Main, :Metal)
        try
            metal_functional = Metal.functional()
            if metal_functional
                Metal.allowscalar(false)
                metal_scores = gnn_node_scores(model, graph; device = :metal)
                all(isfinite, metal_scores) || error("Metal scores must be finite")
                metal_score_count = length(metal_scores)
            end
        catch
            metal_functional = false
            metal_score_count = 0
        end
    end

    (
        node_count = length(nodes.id),
        edge_count = length(edges.source_id),
        feature_rows = size(features, 1),
        feature_cols = size(features, 2),
        score_count = length(scores),
        frontier_rows = length(frontier),
        metal_loaded = status.metal_loaded,
        cuda_loaded = status.cuda_loaded,
        amdgpu_loaded = status.amdgpu_loaded,
        metal_functional = metal_functional,
        metal_score_count = metal_score_count,
    )
end

function timed_probe()
    started = time()
    payload = probe_payload()
    elapsed_ms = (time() - started) * 1_000
    elapsed_ms, payload
end

function percentile(sorted_samples, ratio)
    index = clamp(ceil(Int, length(sorted_samples) * ratio), 1, length(sorted_samples))
    sorted_samples[index]
end

function render_probe_report()
    sample_count = max(
        parse(Int, get(ENV, "WENDAO_GRAPH_GNN_HOST_PROBE_WARM_SAMPLES", "3")),
        1,
    )

    first_ms, first_payload = timed_probe()
    samples = Float64[]
    for _ in 1:sample_count
        elapsed_ms, payload = timed_probe()
        payload.node_count == first_payload.node_count || error("node count changed")
        payload.edge_count == first_payload.edge_count || error("edge count changed")
        payload.feature_rows == first_payload.feature_rows || error("feature row count changed")
        payload.feature_cols == first_payload.feature_cols || error("feature column count changed")
        payload.score_count == first_payload.score_count || error("score count changed")
        payload.frontier_rows == first_payload.frontier_rows || error("frontier row count changed")
        push!(samples, elapsed_ms)
    end
    sorted_samples = sort(samples)

    println(
        "wendaograph_gnn_host_probe " *
        "sample_count=$(sample_count) " *
        "first_ms=$(round(first_ms; digits = 3)) " *
        "warm_min_ms=$(round(sorted_samples[begin]; digits = 3)) " *
        "warm_median_ms=$(round(percentile(sorted_samples, 0.5); digits = 3)) " *
        "warm_p95_ms=$(round(percentile(sorted_samples, 0.95); digits = 3)) " *
        "warm_max_ms=$(round(last(sorted_samples); digits = 3)) " *
        "node_count=$(first_payload.node_count) " *
        "edge_count=$(first_payload.edge_count) " *
        "feature_rows=$(first_payload.feature_rows) " *
        "feature_cols=$(first_payload.feature_cols) " *
        "score_count=$(first_payload.score_count) " *
        "frontier_rows=$(first_payload.frontier_rows) " *
        "metal_loaded=$(first_payload.metal_loaded) " *
        "cuda_loaded=$(first_payload.cuda_loaded) " *
        "amdgpu_loaded=$(first_payload.amdgpu_loaded) " *
        "metal_functional=$(first_payload.metal_functional) " *
        "metal_score_count=$(first_payload.metal_score_count)",
    )
end

render_probe_report()
"#;

/// Timing, shape, and backend diagnostics from one local `WendaoGraph.jl` GNN
/// host probe.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoGraphGnnHostProbeReport {
    /// Number of warm samples measured after the first request.
    pub sample_count: usize,
    /// First host-probe call elapsed milliseconds after Julia package load.
    pub first_ms: f64,
    /// Minimum warm-call elapsed milliseconds.
    pub warm_min_ms: f64,
    /// Median warm-call elapsed milliseconds.
    pub warm_median_ms: f64,
    /// P95 warm-call elapsed milliseconds.
    pub warm_p95_ms: f64,
    /// Maximum warm-call elapsed milliseconds.
    pub warm_max_ms: f64,
    /// Node count in the deterministic probe graph.
    pub node_count: usize,
    /// Edge count in the deterministic probe graph.
    pub edge_count: usize,
    /// Feature matrix row count.
    pub feature_rows: usize,
    /// Feature matrix column count.
    pub feature_cols: usize,
    /// CPU score vector length.
    pub score_count: usize,
    /// Reasoning frontier row count from the GNN score surface.
    pub frontier_rows: usize,
    /// Julia GNN backend load diagnostics.
    pub backend_load: WendaoGraphGnnBackendLoadDiagnostics,
    /// Whether `Metal.functional()` succeeded during the diagnostic probe.
    pub metal_functional: bool,
    /// Metal score vector length when Metal is functional, otherwise zero.
    pub metal_score_count: usize,
}

/// Backend module load diagnostics from one local `WendaoGraph.jl` GNN probe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WendaoGraphGnnBackendLoadDiagnostics {
    /// Whether `Metal.jl` was loaded in the Julia process.
    pub metal_loaded: bool,
    /// Whether `CUDA.jl` was loaded in the Julia process.
    pub cuda_loaded: bool,
    /// Whether `AMDGPU.jl` was loaded in the Julia process.
    pub amdgpu_loaded: bool,
}

/// Runs the local `WendaoGraph.jl` GNN host-process probe.
///
/// # Errors
///
/// Returns an error when the local `WendaoGraph.jl` project cannot be resolved,
/// the Julia process exits unsuccessfully, or the probe output cannot be parsed.
pub fn probe_wendaograph_gnn_host_request(
    warm_sample_count: usize,
) -> Result<WendaoGraphGnnHostProbeReport, String> {
    let julia_project = wendaograph_julia_project()?;
    let output = Command::new("julia")
        .arg(format!("--project={}", julia_project.display()))
        .arg("-e")
        .arg(GNN_HOST_PROBE_JULIA)
        .env(
            WENDAO_GRAPH_GNN_HOST_PROBE_WARM_SAMPLES_ENV,
            warm_sample_count.max(1).to_string(),
        )
        .output()
        .map_err(|error| format!("spawn WendaoGraph GNN host probe: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "WendaoGraph GNN host probe exited with status {}; stderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_gnn_probe_stdout(stdout.as_ref())
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

fn parse_gnn_probe_stdout(stdout: &str) -> Result<WendaoGraphGnnHostProbeReport, String> {
    let line = stdout
        .lines()
        .find(|line| line.starts_with(GNN_HOST_PROBE_PREFIX))
        .ok_or_else(|| {
            format!(
                "WendaoGraph GNN host probe did not print `{GNN_HOST_PROBE_PREFIX}`; stdout:\n{stdout}"
            )
        })?;
    parse_gnn_probe_report_line(line)
}

fn parse_gnn_probe_report_line(line: &str) -> Result<WendaoGraphGnnHostProbeReport, String> {
    let mut fields = HashMap::new();
    for token in line.split_whitespace().skip(1) {
        let (key, value) = token
            .split_once('=')
            .ok_or_else(|| format!("invalid probe token `{token}`"))?;
        fields.insert(key, value);
    }

    Ok(WendaoGraphGnnHostProbeReport {
        sample_count: parse_usize_field(&fields, "sample_count")?,
        first_ms: parse_f64_field(&fields, "first_ms")?,
        warm_min_ms: parse_f64_field(&fields, "warm_min_ms")?,
        warm_median_ms: parse_f64_field(&fields, "warm_median_ms")?,
        warm_p95_ms: parse_f64_field(&fields, "warm_p95_ms")?,
        warm_max_ms: parse_f64_field(&fields, "warm_max_ms")?,
        node_count: parse_usize_field(&fields, "node_count")?,
        edge_count: parse_usize_field(&fields, "edge_count")?,
        feature_rows: parse_usize_field(&fields, "feature_rows")?,
        feature_cols: parse_usize_field(&fields, "feature_cols")?,
        score_count: parse_usize_field(&fields, "score_count")?,
        frontier_rows: parse_usize_field(&fields, "frontier_rows")?,
        backend_load: WendaoGraphGnnBackendLoadDiagnostics {
            metal_loaded: parse_bool_field(&fields, "metal_loaded")?,
            cuda_loaded: parse_bool_field(&fields, "cuda_loaded")?,
            amdgpu_loaded: parse_bool_field(&fields, "amdgpu_loaded")?,
        },
        metal_functional: parse_bool_field(&fields, "metal_functional")?,
        metal_score_count: parse_usize_field(&fields, "metal_score_count")?,
    })
}

fn parse_usize_field(fields: &HashMap<&str, &str>, key: &str) -> Result<usize, String> {
    fields
        .get(key)
        .ok_or_else(|| format!("missing probe field `{key}`"))?
        .parse()
        .map_err(|error| format!("parse probe field `{key}` as usize: {error}"))
}

fn parse_f64_field(fields: &HashMap<&str, &str>, key: &str) -> Result<f64, String> {
    fields
        .get(key)
        .ok_or_else(|| format!("missing probe field `{key}`"))?
        .parse()
        .map_err(|error| format!("parse probe field `{key}` as f64: {error}"))
}

fn parse_bool_field(fields: &HashMap<&str, &str>, key: &str) -> Result<bool, String> {
    match *fields
        .get(key)
        .ok_or_else(|| format!("missing probe field `{key}`"))?
    {
        "true" => Ok(true),
        "false" => Ok(false),
        value => Err(format!("parse probe field `{key}` as bool: `{value}`")),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/integration_support/wendaograph_gnn.rs"]
mod tests;
