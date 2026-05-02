use super::types::RelatedPprKernelTelemetry;
use crate::link_graph::index::ppr::types::RelatedPprComputation;
use crate::link_graph::index::{LinkGraphIndex, LinkGraphPprSubgraphMode, doc_sort_key};
use crate::link_graph::models::LinkGraphRelatedPprDiagnostics;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::time::Instant;

#[allow(clippy::too_many_arguments)]
pub(super) fn finalize_related_ppr_result(
    index: &LinkGraphIndex,
    seed_ids: &HashSet<String>,
    horizon_distances: std::collections::HashMap<String, usize>,
    graph_nodes: &[String],
    alpha: f64,
    max_iter: usize,
    tol: f64,
    subgraph_mode: LinkGraphPprSubgraphMode,
    restrict_to_horizon: bool,
    candidate_count: usize,
    candidate_cap: usize,
    candidate_capped: bool,
    time_budget_ms: f64,
    total_start: &Instant,
    telemetry: &RelatedPprKernelTelemetry,
) -> RelatedPprComputation {
    let partition_max_node_count = telemetry.partition_sizes.iter().copied().max().unwrap_or(0);
    let partition_min_node_count = telemetry.partition_sizes.iter().copied().min().unwrap_or(0);
    let partition_avg_node_count = if telemetry.partition_sizes.is_empty() {
        0.0
    } else {
        usize_to_f64_saturating(telemetry.partition_sizes.iter().sum())
            / usize_to_f64_saturating(telemetry.partition_sizes.len())
    };

    let mut ranked: Vec<(String, usize, f64)> = horizon_distances
        .into_iter()
        .filter(|(doc_id, distance)| *distance > 0 && !seed_ids.contains(doc_id))
        .filter_map(|(doc_id, distance)| {
            telemetry
                .fused_scores_by_doc_id
                .get(&doc_id)
                .copied()
                .map(|score| (doc_id, distance, score))
        })
        .collect();

    ranked.sort_by(|left, right| {
        right
            .2
            .partial_cmp(&left.2)
            .unwrap_or(Ordering::Equal)
            .then(left.1.cmp(&right.1))
            .then_with(|| {
                match (
                    index.docs_by_id.get(&left.0),
                    index.docs_by_id.get(&right.0),
                ) {
                    (Some(a), Some(b)) => doc_sort_key(a).cmp(&doc_sort_key(b)),
                    _ => left.0.cmp(&right.0),
                }
            })
    });

    let diagnostics = LinkGraphRelatedPprDiagnostics {
        alpha,
        max_iter,
        tol,
        iteration_count: telemetry.iteration_count,
        final_residual: telemetry.final_residual,
        candidate_count,
        candidate_cap,
        candidate_capped,
        graph_node_count: graph_nodes.len(),
        subgraph_count: telemetry.subgraph_count,
        partition_max_node_count,
        partition_min_node_count,
        partition_avg_node_count,
        total_duration_ms: total_start.elapsed().as_secs_f64() * 1000.0,
        partition_duration_ms: telemetry.partition_duration_ms,
        kernel_duration_ms: telemetry.kernel_duration_ms,
        fusion_duration_ms: telemetry.fusion_duration_ms,
        subgraph_mode,
        horizon_restricted: restrict_to_horizon,
        time_budget_ms,
        timed_out: telemetry.timed_out,
    };
    RelatedPprComputation {
        ranked_doc_ids: ranked,
        diagnostics,
    }
}

fn usize_to_f64_saturating(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}
