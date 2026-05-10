use std::collections::HashSet;

use crate::search::real_repo_precision::types::{
    RealRepoGoldQuery, RealRepoPrecisionQueryReceipt, RealRepoPrecisionRequiredPathRankReceipt,
};

pub(crate) fn evaluate_gold_query_paths(
    gold_query: &RealRepoGoldQuery,
    observed_paths: Vec<String>,
) -> RealRepoPrecisionQueryReceipt {
    evaluate_gold_query_paths_with_timing(gold_query, observed_paths, 0)
}

pub(crate) fn evaluate_gold_query_paths_with_timing(
    gold_query: &RealRepoGoldQuery,
    observed_paths: Vec<String>,
    query_ms: u128,
) -> RealRepoPrecisionQueryReceipt {
    let observed_paths = observed_paths
        .into_iter()
        .map(|path| normalize_path(path.as_str()))
        .collect::<Vec<_>>();
    let observed_path_set = observed_paths.iter().cloned().collect::<HashSet<_>>();
    let missing_paths = gold_query
        .must_hit_paths
        .iter()
        .map(|path| normalize_path(path.as_str()))
        .filter(|path| !observed_path_set.contains(path))
        .collect::<Vec<_>>();
    let required_path_ranks = required_path_ranks(&gold_query.must_hit_paths, &observed_paths);
    let [recall_one, recall_three, recall_five, recall_ten] = path_recall_bps(&required_path_ranks);
    let mean_required_path_reciprocal_rank_bps = mean_reciprocal_rank_bps(&required_path_ranks);
    let best_required_path_rank = best_required_path_rank(&required_path_ranks);
    let observed_top_path = observed_paths.first().cloned();
    let top_path_matches = gold_query
        .required_top_path
        .as_deref()
        .map(normalize_path)
        .is_none_or(|required| observed_top_path.as_deref() == Some(required.as_str()));
    let passed = missing_paths.is_empty() && top_path_matches;

    RealRepoPrecisionQueryReceipt {
        query_id: gold_query.id.clone(),
        query_kind: gold_query.kind.as_str().to_string(),
        query: gold_query.query.clone(),
        limit: gold_query.limit,
        query_ms,
        passed,
        must_hit_paths: gold_query.must_hit_paths.clone(),
        missing_paths,
        required_top_path: gold_query.required_top_path.clone(),
        observed_top_path,
        required_path_ranks,
        required_path_recall_at_1_bps: recall_one,
        required_path_recall_at_3_bps: recall_three,
        required_path_recall_at_5_bps: recall_five,
        required_path_recall_at_10_bps: recall_ten,
        mean_required_path_reciprocal_rank_bps,
        best_required_path_rank,
        observed_paths,
    }
}

fn path_recall_bps(ranks: &[RealRepoPrecisionRequiredPathRankReceipt]) -> [u32; 4] {
    [
        recall_at_bps(ranks, 1),
        recall_at_bps(ranks, 3),
        recall_at_bps(ranks, 5),
        recall_at_bps(ranks, 10),
    ]
}

fn normalize_path(path: &str) -> String {
    path.trim().replace('\\', "/")
}

fn required_path_ranks(
    required_paths: &[String],
    observed_paths: &[String],
) -> Vec<RealRepoPrecisionRequiredPathRankReceipt> {
    required_paths
        .iter()
        .map(|path| {
            let normalized = normalize_path(path);
            let zero_based_rank = observed_paths
                .iter()
                .position(|observed| observed == &normalized);
            RealRepoPrecisionRequiredPathRankReceipt {
                path: normalized,
                zero_based_rank,
            }
        })
        .collect()
}

fn recall_at_bps(ranks: &[RealRepoPrecisionRequiredPathRankReceipt], k: usize) -> u32 {
    if ranks.is_empty() {
        return 10_000;
    }
    let covered = ranks
        .iter()
        .filter(|rank| rank.zero_based_rank.is_some_and(|value| value < k))
        .count();
    usize_to_u32_saturating((covered * 10_000) / ranks.len())
}

fn mean_reciprocal_rank_bps(ranks: &[RealRepoPrecisionRequiredPathRankReceipt]) -> u32 {
    if ranks.is_empty() {
        return 10_000;
    }
    let total = ranks
        .iter()
        .map(|rank| {
            rank.zero_based_rank.map_or(0, |value| {
                10_000 / usize_to_u32_saturating(value.saturating_add(1))
            })
        })
        .sum::<u32>();
    total / usize_to_u32_saturating(ranks.len())
}

fn best_required_path_rank(ranks: &[RealRepoPrecisionRequiredPathRankReceipt]) -> Option<usize> {
    ranks.iter().filter_map(|rank| rank.zero_based_rank).min()
}

fn usize_to_u32_saturating(value: usize) -> u32 {
    value.try_into().unwrap_or(u32::MAX)
}
