//! Context budget pruning entrypoints and strategy orchestration.

use crate::config::ContextBudgetStrategy;
use crate::session::ChatMessage;

use super::classify::classify_messages_for_budget;
use super::selection::{
    build_budget_candidates, pack_selected_messages, select_candidate_messages,
    select_latest_non_system,
};
use super::types::{ContextBudgetPruneResult, ContextBudgetReport};

#[doc(hidden)]
#[must_use]
pub fn prune_messages_for_token_budget(
    messages: Vec<ChatMessage>,
    budget_tokens: usize,
    reserve_tokens: usize,
) -> Vec<ChatMessage> {
    prune_messages_for_token_budget_with_strategy(
        messages,
        budget_tokens,
        reserve_tokens,
        ContextBudgetStrategy::RecentFirst,
    )
    .messages
}

pub(crate) fn prune_messages_for_token_budget_with_strategy(
    messages: Vec<ChatMessage>,
    budget_tokens: usize,
    reserve_tokens: usize,
    strategy: ContextBudgetStrategy,
) -> ContextBudgetPruneResult {
    let mut report = ContextBudgetReport::new(
        strategy,
        budget_tokens,
        reserve_tokens,
        budget_tokens.saturating_sub(reserve_tokens).max(1),
    );
    if messages.is_empty() {
        report.effective_budget_tokens = if budget_tokens == 0 {
            0
        } else {
            report.effective_budget_tokens
        };
        return ContextBudgetPruneResult { messages, report };
    }

    let effective_budget = if budget_tokens == 0 {
        0
    } else {
        budget_tokens.saturating_sub(reserve_tokens).max(1)
    };
    report.effective_budget_tokens = effective_budget;

    let classified = classify_messages_for_budget(messages, &mut report);

    if effective_budget == 0 {
        return ContextBudgetPruneResult {
            messages: Vec::new(),
            report,
        };
    }

    let mut selected = Vec::new();
    let mut used_tokens = 0usize;
    if let Some(latest_non_system) = select_latest_non_system(&classified.non, effective_budget) {
        used_tokens = used_tokens.saturating_add(latest_non_system.kept_tokens);
        selected.push(latest_non_system);
    }
    let candidates = build_budget_candidates(classified, strategy);
    select_candidate_messages(
        candidates,
        effective_budget,
        &mut used_tokens,
        &mut selected,
    );
    let packed = pack_selected_messages(selected, &mut report);

    ContextBudgetPruneResult {
        messages: packed,
        report,
    }
}
