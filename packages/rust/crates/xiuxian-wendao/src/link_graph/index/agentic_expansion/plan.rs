use super::super::LinkGraphIndex;
use crate::link_graph::agentic::{
    LinkGraphAgenticCandidatePair, LinkGraphAgenticExpansionConfig, LinkGraphAgenticExpansionPlan,
};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::time::Instant;

#[path = "plan/candidates.rs"]
mod candidates;
#[path = "plan/workers.rs"]
mod workers;

use candidates::{agentic_pair_priority, collect_agentic_expansion_candidates, has_direct_edge};
use workers::partition_expansion_workers;

#[derive(Debug, Clone)]
struct ExpansionCandidateDoc {
    doc_id: String,
    rank: f64,
    saliency_signal: f64,
    tags: HashSet<String>,
}

pub(super) fn agentic_expansion_plan_with_config(
    index: &LinkGraphIndex,
    query: Option<&str>,
    config: LinkGraphAgenticExpansionConfig,
) -> LinkGraphAgenticExpansionPlan {
    let normalized_config = config.normalized();
    let normalized_query = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    let started = Instant::now();
    let candidates = collect_agentic_expansion_candidates(
        index,
        normalized_query.as_deref(),
        normalized_config.max_candidates,
    );
    let candidate_notes = candidates.len();
    let total_notes = index.docs_by_id.len();
    let total_possible_pairs =
        candidate_notes.saturating_mul(candidate_notes.saturating_sub(1)) / 2;
    let pair_budget = normalized_config
        .max_workers
        .saturating_mul(normalized_config.max_pairs_per_worker)
        .max(1);

    let mut evaluated_pairs = 0usize;
    let mut timed_out = false;
    let mut ranked_pairs: Vec<LinkGraphAgenticCandidatePair> = Vec::new();

    'outer: for (left_idx, left) in candidates.iter().enumerate() {
        for right in candidates.iter().skip(left_idx + 1) {
            if started.elapsed().as_secs_f64() * 1000.0 >= normalized_config.time_budget_ms {
                timed_out = true;
                break 'outer;
            }
            if has_direct_edge(index, &left.doc_id, &right.doc_id) {
                continue;
            }
            evaluated_pairs = evaluated_pairs.saturating_add(1);
            ranked_pairs.push(LinkGraphAgenticCandidatePair {
                left_id: left.doc_id.clone(),
                right_id: right.doc_id.clone(),
                priority: agentic_pair_priority(left, right),
            });
        }
    }

    ranked_pairs.sort_by(|left, right| {
        right
            .priority
            .partial_cmp(&left.priority)
            .unwrap_or(Ordering::Equal)
            .then(left.left_id.cmp(&right.left_id))
            .then(left.right_id.cmp(&right.right_id))
    });

    let capped_by_pair_limit = ranked_pairs.len() > pair_budget;
    if capped_by_pair_limit {
        ranked_pairs.truncate(pair_budget);
    }

    let workers = partition_expansion_workers(
        &ranked_pairs,
        normalized_config.max_workers,
        normalized_config.max_pairs_per_worker,
    );
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;

    LinkGraphAgenticExpansionPlan {
        query: normalized_query,
        total_notes,
        candidate_notes,
        total_possible_pairs,
        evaluated_pairs,
        selected_pairs: ranked_pairs.len(),
        timed_out,
        capped_by_pair_limit,
        config: normalized_config,
        elapsed_ms,
        workers,
    }
}
