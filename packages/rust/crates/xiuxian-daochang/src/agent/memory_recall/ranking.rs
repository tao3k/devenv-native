use num_traits::ToPrimitive;
use std::time::{SystemTime, UNIX_EPOCH};
use xiuxian_memory_engine::Episode;

use super::{MemoryRecallPlan, RECENCY_HALF_LIFE_HOURS};

/// Keep high-quality recalled episodes according to the dynamic recall plan.
pub(crate) fn filter_recalled_episodes(
    recalled: Vec<(Episode, f32)>,
    plan: &MemoryRecallPlan,
) -> Vec<(Episode, f32)> {
    filter_recalled_episodes_at(recalled, plan, now_unix_ms())
}

pub(crate) fn filter_recalled_episodes_at(
    recalled: Vec<(Episode, f32)>,
    plan: &MemoryRecallPlan,
    now_unix_ms: i64,
) -> Vec<(Episode, f32)> {
    let recency_beta = recency_beta(plan);
    let mut finite = recalled
        .into_iter()
        .filter(|(_, score)| score.is_finite())
        .map(|(episode, score)| {
            let recency = episode_recency_score(&episode, now_unix_ms, RECENCY_HALF_LIFE_HOURS);
            let fused_score = fuse_with_recency(score, recency, recency_beta);
            (episode, fused_score)
        })
        .collect::<Vec<_>>();
    finite.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut selected = finite
        .iter()
        .filter(|(_, score)| *score >= plan.min_score)
        .take(plan.k2)
        .map(|(episode, score)| (episode.clone(), *score))
        .collect::<Vec<_>>();

    // Keep one positive candidate if all were filtered by min-score.
    if selected.is_empty()
        && let Some((episode, score)) = finite.first()
        && *score > 0.0
    {
        selected.push((episode.clone(), *score));
    }

    selected
}

fn recency_beta(plan: &MemoryRecallPlan) -> f32 {
    if plan.budget_pressure >= 1.0 {
        0.28
    } else if plan.budget_pressure >= 0.8 {
        0.24
    } else if plan.window_pressure >= 0.75 {
        0.18
    } else {
        0.14
    }
}

fn episode_recency_score(episode: &Episode, now_unix_ms: i64, half_life_hours: f32) -> f32 {
    if !half_life_hours.is_finite() || half_life_hours <= 0.0 {
        return 1.0;
    }
    let age_ms = now_unix_ms
        .saturating_sub(episode.created_at)
        .max(0)
        .to_f32()
        .unwrap_or(f32::MAX);
    let age_hours = age_ms / (1000.0 * 60.0 * 60.0);
    let exponent = -(std::f32::consts::LN_2 * age_hours / half_life_hours);
    exponent.exp().clamp(0.0, 1.0)
}

fn fuse_with_recency(base_score: f32, recency_score: f32, recency_beta: f32) -> f32 {
    let beta = recency_beta.clamp(0.0, 0.9);
    ((1.0 - beta) * base_score + beta * recency_score).clamp(-1.0, 1.0)
}

fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
        })
}
