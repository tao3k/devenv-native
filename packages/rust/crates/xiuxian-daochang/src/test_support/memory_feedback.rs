//! Memory-recall feedback helpers exposed for integration tests.

use super::memory_recall::MemoryRecallPlan;
use crate::agent::{memory_recall, memory_recall_feedback};

pub use xiuxian_memory_engine::RecallFeedbackOutcome as FeedbackOutcome;

/// User-originated recall feedback source token.
pub const RECALL_FEEDBACK_SOURCE_USER: &str = memory_recall_feedback::RECALL_FEEDBACK_SOURCE_USER;
/// Tool-originated recall feedback source token.
pub const RECALL_FEEDBACK_SOURCE_TOOL: &str = memory_recall_feedback::RECALL_FEEDBACK_SOURCE_TOOL;
/// Assistant-originated recall feedback source token.
pub const RECALL_FEEDBACK_SOURCE_ASSISTANT: &str =
    memory_recall_feedback::RECALL_FEEDBACK_SOURCE_ASSISTANT;
/// Command-originated recall feedback source token.
pub const RECALL_FEEDBACK_SOURCE_COMMAND: &str =
    memory_recall_feedback::RECALL_FEEDBACK_SOURCE_COMMAND;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Test-facing summary of tool execution outcomes.
pub struct ToolExecutionSummary {
    pub attempted: u32,
    pub succeeded: u32,
    pub failed: u32,
}

impl ToolExecutionSummary {
    /// Records one tool execution result.
    pub fn record_result(&mut self, is_error: bool) {
        let mut internal = to_internal_summary(*self);
        internal.record_result(is_error);
        *self = from_internal_summary(internal);
    }

    /// Records one transport-level tool failure.
    pub fn record_transport_failure(&mut self) {
        let mut internal = to_internal_summary(*self);
        internal.record_transport_failure();
        *self = from_internal_summary(internal);
    }

    #[must_use]
    /// Infers a recall feedback outcome from tool execution counts.
    pub fn inferred_outcome(self) -> Option<FeedbackOutcome> {
        to_internal_summary(self).inferred_outcome()
    }
}

#[must_use]
/// Updates recall feedback bias after an observed outcome.
pub fn update_feedback_bias(previous: f32, outcome: FeedbackOutcome) -> f32 {
    memory_recall_feedback::update_feedback_bias(previous, outcome)
}

#[must_use]
/// Classifies an assistant message as a recall feedback outcome.
pub fn classify_assistant_outcome(message: &str) -> FeedbackOutcome {
    memory_recall_feedback::classify_assistant_outcome(message)
}

#[must_use]
/// Parses explicit user recall feedback from a message.
pub fn parse_explicit_user_feedback(message: &str) -> Option<FeedbackOutcome> {
    memory_recall_feedback::parse_explicit_user_feedback(message)
}

#[must_use]
/// Resolves the final recall feedback outcome and source.
pub fn resolve_feedback_outcome(
    user_message: &str,
    tool_summary: Option<&ToolExecutionSummary>,
    assistant_message: &str,
) -> (FeedbackOutcome, &'static str) {
    let internal_summary = tool_summary.map(|summary| to_internal_summary(*summary));
    memory_recall_feedback::resolve_feedback_outcome(
        user_message,
        internal_summary.as_ref(),
        assistant_message,
    )
}

#[must_use]
/// Applies recall feedback bias to a recall plan.
pub fn apply_feedback_to_plan(plan: MemoryRecallPlan, feedback_bias: f32) -> MemoryRecallPlan {
    let internal =
        memory_recall_feedback::apply_feedback_to_plan(to_internal_plan(plan), feedback_bias);
    from_internal_plan(internal)
}

fn to_internal_summary(
    summary: ToolExecutionSummary,
) -> memory_recall_feedback::ToolExecutionSummary {
    memory_recall_feedback::ToolExecutionSummary {
        attempted: summary.attempted,
        succeeded: summary.succeeded,
        failed: summary.failed,
    }
}

fn from_internal_summary(
    summary: memory_recall_feedback::ToolExecutionSummary,
) -> ToolExecutionSummary {
    ToolExecutionSummary {
        attempted: summary.attempted,
        succeeded: summary.succeeded,
        failed: summary.failed,
    }
}

fn to_internal_plan(plan: MemoryRecallPlan) -> memory_recall::MemoryRecallPlan {
    memory_recall::MemoryRecallPlan {
        k1: plan.k1,
        k2: plan.k2,
        lambda: plan.lambda,
        min_score: plan.min_score,
        max_context_chars: plan.max_context_chars,
        budget_pressure: plan.budget_pressure,
        window_pressure: plan.window_pressure,
        effective_budget_tokens: plan.effective_budget_tokens,
    }
}

fn from_internal_plan(plan: memory_recall::MemoryRecallPlan) -> MemoryRecallPlan {
    MemoryRecallPlan {
        k1: plan.k1,
        k2: plan.k2,
        lambda: plan.lambda,
        min_score: plan.min_score,
        max_context_chars: plan.max_context_chars,
        budget_pressure: plan.budget_pressure,
        window_pressure: plan.window_pressure,
        effective_budget_tokens: plan.effective_budget_tokens,
    }
}
