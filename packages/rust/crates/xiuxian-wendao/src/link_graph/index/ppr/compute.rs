#[path = "compute/finalize.rs"]
mod finalize;
#[path = "compute/orchestrate.rs"]
mod orchestrate;

use self::finalize::finalize_related_ppr_result;
use self::orchestrate::run_related_ppr_orchestration;
use crate::link_graph::index::ppr::runtime::resolve_related_ppr_runtime;
use crate::link_graph::index::ppr::types::RelatedPprComputation;
use crate::link_graph::index::{
    LinkGraphIndex, LinkGraphPprSubgraphMode, LinkGraphRelatedPprOptions,
};
use crate::link_graph::runtime_config::resolve_link_graph_related_runtime;
use crate::link_graph::saliency::{learned_saliency_signal_from_state, valkey_saliency_get_many};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct RelatedPprKernelTelemetry {
    fused_scores_by_doc_id: HashMap<String, f64>,
    iteration_count: usize,
    final_residual: f64,
    subgraph_count: usize,
    partition_sizes: Vec<usize>,
    partition_duration_ms: f64,
    kernel_duration_ms: f64,
    fusion_duration_ms: f64,
    timed_out: bool,
}

impl LinkGraphIndex {
    pub(in crate::link_graph::index) fn related_ppr_compute(
        &self,
        seed_ids: &HashSet<String>,
        max_distance: usize,
        options: Option<&LinkGraphRelatedPprOptions>,
    ) -> Option<RelatedPprComputation> {
        let total_start = Instant::now();
        if seed_ids.is_empty() {
            return None;
        }
        let runtime = resolve_link_graph_related_runtime();
        let candidate_cap = runtime.max_candidates.max(1);
        let max_partitions = runtime.max_partitions.max(1);
        let time_budget_ms = runtime.time_budget_ms.max(1.0);
        let budget_duration = Duration::from_secs_f64(time_budget_ms / 1000.0);
        let deadline = Some(total_start + budget_duration);

        let bounded_distance = max_distance.max(1);
        let raw_horizon_distances =
            self.collect_bidirectional_distance_map(seed_ids, bounded_distance);
        if raw_horizon_distances.is_empty() {
            return None;
        }
        let raw_candidate_count =
            Self::candidate_count_from_horizon(&raw_horizon_distances, seed_ids);
        let candidate_capped = raw_candidate_count > candidate_cap;
        let horizon_distances = if candidate_capped {
            self.trim_horizon_candidates(&raw_horizon_distances, seed_ids, candidate_cap)
        } else {
            raw_horizon_distances
        };

        let (alpha, max_iter, tol, subgraph_mode) = resolve_related_ppr_runtime(options);
        let restrict_to_horizon = match subgraph_mode {
            LinkGraphPprSubgraphMode::Disabled => false,
            LinkGraphPprSubgraphMode::Force => true,
            LinkGraphPprSubgraphMode::Auto => horizon_distances.len() < self.docs_by_id.len(),
        };

        let graph_nodes =
            self.build_graph_nodes_for_related_ppr(&horizon_distances, restrict_to_horizon);
        if graph_nodes.is_empty() {
            return None;
        }
        let candidate_count = Self::candidate_count_from_horizon(&horizon_distances, seed_ids);
        let seed_weights = resolve_related_ppr_seed_weights(seed_ids);
        let telemetry = run_related_ppr_orchestration(
            self,
            &seed_weights,
            &graph_nodes,
            bounded_distance,
            alpha,
            max_iter,
            tol,
            subgraph_mode,
            restrict_to_horizon,
            max_partitions,
            deadline,
        )?;

        Some(finalize_related_ppr_result(
            self,
            seed_ids,
            horizon_distances,
            &graph_nodes,
            alpha,
            max_iter,
            tol,
            subgraph_mode,
            restrict_to_horizon,
            candidate_count,
            candidate_cap,
            candidate_capped,
            time_budget_ms,
            &total_start,
            &telemetry,
        ))
    }

    pub(in crate::link_graph::index) fn related_ppr_ranked_doc_ids(
        &self,
        seed_ids: &HashSet<String>,
        max_distance: usize,
        options: Option<&LinkGraphRelatedPprOptions>,
    ) -> Vec<(String, usize, f64)> {
        self.related_ppr_compute(seed_ids, max_distance, options)
            .map(|row| row.ranked_doc_ids)
            .unwrap_or_default()
    }
}

fn resolve_related_ppr_seed_weights(seed_ids: &HashSet<String>) -> HashMap<String, f64> {
    let mut ordered_seed_ids: Vec<String> = seed_ids.iter().cloned().collect();
    ordered_seed_ids.sort_unstable();
    let fallback_weights = ordered_seed_ids
        .iter()
        .cloned()
        .map(|seed_id| (seed_id, 1.0))
        .collect::<HashMap<_, _>>();

    let Ok(states) = valkey_saliency_get_many(&ordered_seed_ids) else {
        return fallback_weights;
    };

    let mut total_weight = 0.0_f64;
    let mut weighted_seed_ids: HashMap<String, f64> =
        HashMap::with_capacity(ordered_seed_ids.len());
    for seed_id in ordered_seed_ids {
        let weight = states
            .get(&seed_id)
            .map_or(1.0, learned_saliency_signal_from_state)
            .max(0.0);
        total_weight += weight;
        weighted_seed_ids.insert(seed_id, weight);
    }

    if total_weight <= 0.0 {
        fallback_weights
    } else {
        weighted_seed_ids
    }
}
