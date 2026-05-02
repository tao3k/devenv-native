use crate::{
    ContextBudgetStrategy, RuntimeSettings,
    env_parse::{parse_bool_from_env, parse_positive_u32_from_env, parse_positive_usize_from_env},
};
use anyhow::{Result, anyhow};

use super::env_lookup::non_empty_env;
use super::types::SessionRuntimeOptions;

fn parse_context_budget_strategy(raw: &str, source: &str) -> Result<ContextBudgetStrategy> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "recent_first" => Ok(ContextBudgetStrategy::RecentFirst),
        "summary_first" => Ok(ContextBudgetStrategy::SummaryFirst),
        _ => Err(anyhow!(
            "invalid {source}: '{raw}' (expected one of: recent_first, summary_first)"
        )),
    }
}

fn resolve_context_budget_strategy(
    runtime_settings: &RuntimeSettings,
) -> Result<ContextBudgetStrategy> {
    if let Some(raw) = non_empty_env("XIUXIAN_DAOCHANG_CONTEXT_BUDGET_STRATEGY") {
        return parse_context_budget_strategy(&raw, "XIUXIAN_DAOCHANG_CONTEXT_BUDGET_STRATEGY");
    }

    if let Some(raw) = runtime_settings
        .session
        .context_budget_strategy
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return parse_context_budget_strategy(raw, "session.context_budget_strategy");
    }

    Ok(ContextBudgetStrategy::RecentFirst)
}

pub(super) fn resolve_runtime_session_options(
    runtime_settings: &RuntimeSettings,
) -> Result<SessionRuntimeOptions> {
    let window_max_turns = parse_positive_usize_from_env("XIUXIAN_DAOCHANG_WINDOW_MAX_TURNS")
        .or(runtime_settings
            .session
            .window_max_turns
            .filter(|value| *value > 0))
        .or(Some(256));
    let consolidation_take_turns =
        parse_positive_usize_from_env("XIUXIAN_DAOCHANG_CONSOLIDATION_TAKE_TURNS")
            .or(runtime_settings
                .session
                .consolidation_take_turns
                .filter(|value| *value > 0))
            .unwrap_or(32);

    Ok(SessionRuntimeOptions {
        max_tool_rounds: parse_positive_u32_from_env("XIUXIAN_DAOCHANG_MAX_TOOL_ROUNDS")
            .or(runtime_settings.telegram.max_tool_rounds)
            .unwrap_or(30),
        window_max_turns,
        consolidation_threshold_turns: parse_positive_usize_from_env(
            "XIUXIAN_DAOCHANG_CONSOLIDATION_THRESHOLD_TURNS",
        )
        .or(runtime_settings
            .session
            .consolidation_threshold_turns
            .filter(|value| *value > 0))
        .or_else(|| window_max_turns.map(|max_turns| (max_turns.saturating_mul(3)) / 4)),
        consolidation_take_turns,
        consolidation_async: parse_bool_from_env("XIUXIAN_DAOCHANG_CONSOLIDATION_ASYNC")
            .or(runtime_settings.session.consolidation_async)
            .unwrap_or(true),
        context_budget_tokens: parse_positive_usize_from_env(
            "XIUXIAN_DAOCHANG_CONTEXT_BUDGET_TOKENS",
        )
        .or(runtime_settings
            .session
            .context_budget_tokens
            .filter(|value| *value > 0))
        .or(Some(6000)),
        context_budget_reserve_tokens: parse_positive_usize_from_env(
            "XIUXIAN_DAOCHANG_CONTEXT_BUDGET_RESERVE_TOKENS",
        )
        .or(runtime_settings
            .session
            .context_budget_reserve_tokens
            .filter(|value| *value > 0))
        .unwrap_or(512),
        context_budget_strategy: resolve_context_budget_strategy(runtime_settings)?,
        summary_max_segments: parse_positive_usize_from_env(
            "XIUXIAN_DAOCHANG_SUMMARY_MAX_SEGMENTS",
        )
        .or(runtime_settings
            .session
            .summary_max_segments
            .filter(|value| *value > 0))
        .unwrap_or(8),
        summary_max_chars: parse_positive_usize_from_env("XIUXIAN_DAOCHANG_SUMMARY_MAX_CHARS")
            .or(runtime_settings
                .session
                .summary_max_chars
                .filter(|value| *value > 0))
            .unwrap_or(480),
    })
}
