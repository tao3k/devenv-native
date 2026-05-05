//! Memory recall planning types and constants.

/// System message name used for injected memory recall context.
pub(crate) const MEMORY_RECALL_MESSAGE_NAME: &str = "agent.memory.recall";
pub(crate) const RECENCY_HALF_LIFE_HOURS: f32 = 24.0 * 7.0;

#[derive(Debug, Clone, Copy)]
pub(crate) struct MemoryRecallInput {
    pub base_k1: usize,
    pub base_k2: usize,
    pub base_lambda: f32,
    pub context_budget_tokens: Option<usize>,
    pub context_budget_reserve_tokens: usize,
    pub context_tokens_before_recall: usize,
    pub active_turns_estimate: usize,
    pub window_max_turns: Option<usize>,
    pub summary_segment_count: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MemoryRecallPlan {
    pub k1: usize,
    pub k2: usize,
    pub lambda: f32,
    pub min_score: f32,
    pub max_context_chars: usize,
    pub budget_pressure: f32,
    pub window_pressure: f32,
    pub effective_budget_tokens: Option<usize>,
}
