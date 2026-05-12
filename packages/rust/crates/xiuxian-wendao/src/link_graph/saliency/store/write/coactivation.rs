use super::touch::apply_touch_with_connection;
use super::types::{
    CoactivationNeighbor, CoactivationNeighborDirection, CoactivationPropagationTarget,
    TouchUpdateSpec,
};
use crate::link_graph::runtime_config::resolve_link_graph_coactivation_runtime;
use crate::link_graph::saliency::LinkGraphSaliencyPolicy;
use crate::link_graph::saliency::{edge_in_key, edge_out_key};
use std::collections::{HashMap, HashSet};

const OUTBOUND_COACTIVATION_DIRECTION_SCALE: f64 = 1.0;
const INBOUND_COACTIVATION_DIRECTION_SCALE: f64 = 0.25;

pub(super) fn direct_coactivation_neighbors(
    conn: &mut redis::Connection,
    node_id: &str,
    key_prefix: &str,
    max_neighbors_per_direction: usize,
) -> Vec<CoactivationNeighbor> {
    if max_neighbors_per_direction == 0 {
        return Vec::new();
    }

    let outbound_key = edge_out_key(node_id, key_prefix);
    let mut inbound_neighbors = redis::cmd("SMEMBERS")
        .arg(edge_in_key(node_id, key_prefix))
        .query::<Vec<String>>(conn)
        .unwrap_or_default();
    inbound_neighbors.sort_unstable();
    let outbound_limit =
        isize::try_from(max_neighbors_per_direction.saturating_sub(1)).unwrap_or(isize::MAX);
    let outbound_neighbors = redis::cmd("ZREVRANGE")
        .arg(&outbound_key)
        .arg(0)
        .arg(outbound_limit)
        .query::<Vec<String>>(conn)
        .unwrap_or_default();

    let mut seen: HashSet<String> = HashSet::new();
    let mut neighbors: Vec<CoactivationNeighbor> = Vec::new();
    for (rank, neighbor) in outbound_neighbors.into_iter().enumerate() {
        let trimmed = neighbor.trim();
        if trimmed.is_empty() || trimmed == node_id {
            continue;
        }
        if !seen.insert(trimmed.to_string()) {
            continue;
        }
        neighbors.push(CoactivationNeighbor {
            node_id: trimmed.to_string(),
            direction: CoactivationNeighborDirection::Outbound,
            rank,
        });
    }

    for (rank, neighbor) in inbound_neighbors.into_iter().enumerate() {
        let trimmed = neighbor.trim();
        if trimmed.is_empty() || trimmed == node_id {
            continue;
        }
        if !seen.insert(trimmed.to_string()) {
            continue;
        }
        neighbors.push(CoactivationNeighbor {
            node_id: trimmed.to_string(),
            direction: CoactivationNeighborDirection::Inbound,
            rank,
        });
        if neighbors.len() >= max_neighbors_per_direction.saturating_mul(2) {
            break;
        }
    }
    neighbors
}

pub(super) fn coactivation_weight_for_neighbor(neighbor: &CoactivationNeighbor) -> f64 {
    let rank_f64 = f64::from(u32::try_from(neighbor.rank).unwrap_or(u32::MAX));
    let rank_scale = 1.0 / (rank_f64 + 1.0);
    let direction_scale = match neighbor.direction {
        CoactivationNeighborDirection::Outbound => OUTBOUND_COACTIVATION_DIRECTION_SCALE,
        CoactivationNeighborDirection::Inbound => INBOUND_COACTIVATION_DIRECTION_SCALE,
    };
    rank_scale * direction_scale
}

pub(super) fn bounded_coactivation_targets(
    conn: &mut redis::Connection,
    node_id: &str,
    key_prefix: &str,
    runtime: &crate::link_graph::runtime_config::models::LinkGraphCoactivationRuntimeConfig,
) -> Vec<CoactivationPropagationTarget> {
    let max_hops = runtime.max_hops.clamp(1, 2);
    let total_budget = runtime.max_total_propagations;
    if total_budget == 0 {
        return Vec::new();
    }

    let mut seen: HashSet<String> = HashSet::from([node_id.to_string()]);
    let mut targets: Vec<CoactivationPropagationTarget> = Vec::new();
    let mut frontier: Vec<(String, f64)> = vec![(node_id.to_string(), 1.0)];

    for hop in 1..=max_hops {
        if frontier.is_empty() || targets.len() >= total_budget {
            break;
        }

        let next_frontier_weights =
            next_coactivation_frontier_weights(conn, frontier, hop, key_prefix, runtime, &seen);

        let mut hop_targets: Vec<CoactivationPropagationTarget> = next_frontier_weights
            .into_iter()
            .map(|(target_node_id, weight)| CoactivationPropagationTarget {
                node_id: target_node_id,
                hop,
                weight,
            })
            .collect();
        hop_targets.sort_by(|left, right| {
            right
                .weight
                .partial_cmp(&left.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.node_id.cmp(&right.node_id))
        });

        let remaining_budget = total_budget.saturating_sub(targets.len());
        if remaining_budget == 0 {
            break;
        }
        hop_targets.truncate(remaining_budget);

        for target in &hop_targets {
            seen.insert(target.node_id.clone());
        }
        frontier = hop_targets
            .iter()
            .map(|target| (target.node_id.clone(), target.weight))
            .collect();
        targets.extend(hop_targets);
    }

    targets
}

fn next_coactivation_frontier_weights(
    conn: &mut redis::Connection,
    frontier: Vec<(String, f64)>,
    hop: usize,
    key_prefix: &str,
    runtime: &crate::link_graph::runtime_config::models::LinkGraphCoactivationRuntimeConfig,
    seen: &HashSet<String>,
) -> HashMap<String, f64> {
    frontier
        .into_iter()
        .flat_map(|(source_node_id, source_weight)| {
            direct_coactivation_neighbors(
                conn,
                &source_node_id,
                key_prefix,
                runtime.max_neighbors_per_direction,
            )
            .into_iter()
            .filter_map(move |neighbor| {
                coactivation_hop_weight(&neighbor, source_weight, hop, runtime)
                    .map(|weight| (neighbor.node_id, weight))
            })
        })
        .filter(|(node_id, _)| !seen.contains(node_id))
        .fold(HashMap::new(), |mut weights, (node_id, hop_weight)| {
            weights
                .entry(node_id)
                .and_modify(|weight: &mut f64| *weight = weight.max(hop_weight))
                .or_insert(hop_weight);
            weights
        })
}

fn coactivation_hop_weight(
    neighbor: &CoactivationNeighbor,
    source_weight: f64,
    hop: usize,
    runtime: &crate::link_graph::runtime_config::models::LinkGraphCoactivationRuntimeConfig,
) -> Option<f64> {
    let edge_weight = coactivation_weight_for_neighbor(neighbor);
    if edge_weight <= f64::EPSILON {
        return None;
    }
    let hop_weight = if hop <= 1 {
        source_weight * edge_weight
    } else {
        source_weight * runtime.hop_decay_scale * edge_weight
    };
    (hop_weight > f64::EPSILON).then_some(hop_weight)
}

pub(super) fn propagate_coactivation(
    conn: &mut redis::Connection,
    node_id: &str,
    key_prefix: &str,
    now_unix: i64,
    policy: LinkGraphSaliencyPolicy,
) {
    let runtime = resolve_link_graph_coactivation_runtime();
    if !runtime.enabled {
        return;
    }

    let scaled_alpha = policy.alpha * runtime.alpha_scale;
    if scaled_alpha <= f64::EPSILON {
        return;
    }

    let propagated_policy = LinkGraphSaliencyPolicy {
        alpha: scaled_alpha,
        minimum: policy.minimum,
        maximum: policy.maximum,
    }
    .normalized();
    let propagated_spec = TouchUpdateSpec {
        activation_delta: 1,
        saliency_base: None,
        decay_rate_override: None,
        policy: propagated_policy,
        now_unix,
    };

    for target in bounded_coactivation_targets(conn, node_id, key_prefix, &runtime) {
        let alpha_scale = target.weight;
        if alpha_scale <= f64::EPSILON {
            continue;
        }
        let weighted_policy = LinkGraphSaliencyPolicy {
            alpha: propagated_spec.policy.alpha * alpha_scale,
            ..propagated_spec.policy
        }
        .normalized();
        let weighted_spec = TouchUpdateSpec {
            policy: weighted_policy,
            ..propagated_spec
        };
        let neighbor_id = target.node_id;
        if let Err(error) =
            apply_touch_with_connection(conn, &neighbor_id, key_prefix, weighted_spec)
        {
            log::warn!(
                "Failed to propagate co-activation from '{node_id}' to '{neighbor_id}' at hop {hop}: {error}",
                hop = target.hop
            );
        }
    }
}
