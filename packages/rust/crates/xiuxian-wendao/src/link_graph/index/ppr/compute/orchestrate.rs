use super::RelatedPprKernelTelemetry;
use crate::link_graph::index::ppr::types::RelatedPprKernelResult;
use crate::link_graph::index::{LinkGraphIndex, LinkGraphPprSubgraphMode};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[allow(clippy::too_many_arguments)]
pub(super) fn run_related_ppr_orchestration(
    index: &LinkGraphIndex,
    seeds: &HashMap<String, f64>,
    graph_nodes: &[String],
    bounded_distance: usize,
    alpha: f64,
    max_iter: usize,
    tol: f64,
    subgraph_mode: LinkGraphPprSubgraphMode,
    restrict_to_horizon: bool,
    max_partitions: usize,
    deadline: Option<Instant>,
) -> Option<RelatedPprKernelTelemetry> {
    let mut fused_scores_by_doc_id: HashMap<String, f64> = HashMap::new();
    let mut iteration_count = 0_usize;
    let mut final_residual = 0.0_f64;
    let mut subgraph_count = 0_usize;
    let mut partition_sizes: Vec<usize> = Vec::new();
    let mut partition_duration_ms = 0.0_f64;
    let mut kernel_duration_ms = 0.0_f64;
    let mut fusion_duration_ms = 0.0_f64;
    let mut timed_out = false;

    let seed_ids: HashSet<String> = seeds.keys().cloned().collect();
    let mut should_partition = LinkGraphIndex::should_partition_related_ppr(
        subgraph_mode,
        restrict_to_horizon,
        graph_nodes.len(),
        seeds.len(),
    );
    if should_partition && LinkGraphIndex::deadline_exceeded(deadline) {
        timed_out = true;
        should_partition = false;
    }
    if should_partition {
        let partition_start = Instant::now();
        let universe: HashSet<String> = graph_nodes.iter().cloned().collect();
        let partitions = index.build_related_ppr_partitions(
            &seed_ids,
            bounded_distance,
            &universe,
            max_partitions,
        );
        partition_duration_ms = partition_start.elapsed().as_secs_f64() * 1000.0;
        partition_sizes = partitions.iter().map(Vec::len).collect();

        let kernel_start = Instant::now();
        let kernels: Vec<RelatedPprKernelResult> = partitions
            .par_iter()
            .filter_map(|partition_nodes| {
                index.run_related_ppr_kernel(partition_nodes, seeds, alpha, max_iter, tol, deadline)
            })
            .collect();
        kernel_duration_ms = kernel_start.elapsed().as_secs_f64() * 1000.0;
        if LinkGraphIndex::deadline_exceeded(deadline) {
            timed_out = true;
        }

        let fusion_start = Instant::now();
        for kernel in kernels {
            subgraph_count += 1;
            iteration_count = iteration_count.max(kernel.iteration_count);
            final_residual = final_residual.max(kernel.final_residual);
            timed_out |= kernel.timed_out;
            for (doc_id, score) in kernel.scores_by_doc_id {
                let current = fused_scores_by_doc_id.entry(doc_id).or_insert(0.0);
                *current = current.max(score);
            }
        }
        fusion_duration_ms = fusion_start.elapsed().as_secs_f64() * 1000.0;
    }

    if subgraph_count == 0 {
        let kernel_start = Instant::now();
        let kernel =
            index.run_related_ppr_kernel(graph_nodes, seeds, alpha, max_iter, tol, deadline)?;
        kernel_duration_ms = kernel_start.elapsed().as_secs_f64() * 1000.0;
        subgraph_count = 1;
        iteration_count = kernel.iteration_count;
        final_residual = kernel.final_residual;
        timed_out |= kernel.timed_out;
        fused_scores_by_doc_id = kernel.scores_by_doc_id;
        partition_sizes = vec![graph_nodes.len()];
    }

    Some(RelatedPprKernelTelemetry {
        fused_scores_by_doc_id,
        iteration_count,
        final_residual,
        subgraph_count,
        partition_sizes,
        partition_duration_ms,
        kernel_duration_ms,
        fusion_duration_ms,
        timed_out,
    })
}
