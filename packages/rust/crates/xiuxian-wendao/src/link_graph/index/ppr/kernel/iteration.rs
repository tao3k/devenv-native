use crate::link_graph::index::LinkGraphIndex;
use std::collections::HashMap;
use std::time::Instant;

use super::types::{KernelIterationOutcome, RestartState};

pub(crate) fn build_restart_state(
    graph_node_count: usize,
    seeds: &HashMap<String, f64>,
    node_to_idx: &HashMap<String, usize>,
) -> Option<RestartState> {
    let weighted_seeds = seeds
        .iter()
        .filter_map(|(seed_id, &weight)| {
            node_to_idx
                .get(seed_id)
                .copied()
                .map(|seed_idx| (seed_idx, weight.max(0.0)))
        })
        .collect::<Vec<_>>();
    let total_seed_weight = weighted_seeds
        .iter()
        .map(|(_, weight)| *weight)
        .sum::<f64>();

    if total_seed_weight <= 0.0 {
        return None;
    }

    let mut teleport = vec![0.0_f64; graph_node_count];
    weighted_seeds
        .iter()
        .for_each(|(seed_idx, weight)| teleport[*seed_idx] = *weight / total_seed_weight);
    let restart_nodes = teleport
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, value)| *value > 0.0)
        .collect::<Vec<_>>();

    Some(RestartState {
        teleport,
        restart_nodes,
    })
}

pub(crate) fn run_kernel_iterations(
    adjacency: &[Vec<usize>],
    restart_state: &RestartState,
    alpha: f64,
    max_iter: usize,
    tol: f64,
    deadline: Option<Instant>,
) -> KernelIterationOutcome {
    let mut scores = restart_state.teleport.clone();
    let mut next_scores = vec![0.0_f64; adjacency.len()];
    let mut iteration_count = 0_usize;
    let mut final_residual = 0.0_f64;
    let mut timed_out = false;

    for _ in 0..max_iter {
        next_scores.fill(0.0);
        let restart_scale = (1.0 - alpha).clamp(0.0, 1.0);
        for (idx, restart) in &restart_state.restart_nodes {
            next_scores[*idx] = restart_scale * *restart;
        }

        let mut dangling_mass = 0.0_f64;
        for (source_idx, outgoing) in adjacency.iter().enumerate() {
            let source_score = scores[source_idx];
            if source_score <= 0.0 {
                continue;
            }
            if outgoing.is_empty() {
                dangling_mass += source_score;
                continue;
            }
            let step = alpha * source_score / usize_to_f64_saturating(outgoing.len());
            for &target_idx in outgoing {
                next_scores[target_idx] += step;
            }
        }

        if dangling_mass > 0.0 {
            let leak = alpha * dangling_mass;
            for (idx, restart) in &restart_state.restart_nodes {
                next_scores[*idx] += leak * *restart;
            }
        }

        let residual: f64 = next_scores
            .iter()
            .zip(scores.iter())
            .map(|(next, current)| (next - current).abs())
            .sum();
        iteration_count += 1;
        final_residual = residual;
        std::mem::swap(&mut scores, &mut next_scores);
        if residual <= tol {
            break;
        }
        if LinkGraphIndex::deadline_exceeded(deadline) {
            timed_out = true;
            break;
        }
    }

    KernelIterationOutcome {
        scores,
        iteration_count,
        final_residual,
        timed_out,
    }
}

fn usize_to_f64_saturating(value: usize) -> f64 {
    u32::try_from(value).map_or(f64::from(u32::MAX), f64::from)
}
