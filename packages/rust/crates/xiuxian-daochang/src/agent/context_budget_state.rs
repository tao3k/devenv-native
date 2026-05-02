//! Context budget telemetry and reset-window persistence.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::ContextBudgetStrategy;

use super::Agent;
use super::context_budget::{ContextBudgetClassStats, ContextBudgetReport};

/// Snapshot of class-level context-budget accounting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionContextBudgetClassSnapshot {
    /// Input messages in this class before pruning.
    pub input_messages: usize,
    /// Messages kept in this class after pruning.
    pub kept_messages: usize,
    /// Messages dropped in this class.
    pub dropped_messages: usize,
    /// Messages truncated in this class.
    pub truncated_messages: usize,
    /// Input tokens in this class before pruning.
    pub input_tokens: usize,
    /// Tokens kept in this class after pruning.
    pub kept_tokens: usize,
    /// Tokens dropped in this class.
    pub dropped_tokens: usize,
    /// Tokens truncated in this class.
    pub truncated_tokens: usize,
}

impl SessionContextBudgetClassSnapshot {
    fn from_stats(stats: &ContextBudgetClassStats) -> Self {
        Self {
            input_messages: stats.input_messages,
            kept_messages: stats.kept_messages,
            dropped_messages: stats.dropped_messages(),
            truncated_messages: stats.truncated_messages,
            input_tokens: stats.input_tokens,
            kept_tokens: stats.kept_tokens,
            dropped_tokens: stats.dropped_tokens(),
            truncated_tokens: stats.truncated_tokens,
        }
    }
}

/// Session-level context-budget snapshot for one pruning report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionContextBudgetSnapshot {
    /// Snapshot creation time in Unix milliseconds.
    pub created_at_unix_ms: u64,
    /// Applied budget strategy.
    pub strategy: ContextBudgetStrategy,
    /// Configured budget tokens.
    pub budget_tokens: usize,
    /// Configured reserve tokens.
    pub reserve_tokens: usize,
    /// Effective budget tokens after reserve.
    pub effective_budget_tokens: usize,
    /// Total messages before pruning.
    pub pre_messages: usize,
    /// Total messages after pruning.
    pub post_messages: usize,
    /// Dropped messages (`pre - post`).
    pub dropped_messages: usize,
    /// Total tokens before pruning.
    pub pre_tokens: usize,
    /// Total tokens after pruning.
    pub post_tokens: usize,
    /// Dropped tokens (`pre - post`).
    pub dropped_tokens: usize,
    /// Snapshot for non-system messages.
    pub non_system: SessionContextBudgetClassSnapshot,
    /// Snapshot for regular system messages.
    pub regular_system: SessionContextBudgetClassSnapshot,
    /// Snapshot for summary system messages.
    pub summary_system: SessionContextBudgetClassSnapshot,
}

impl SessionContextBudgetSnapshot {
    pub(crate) fn from_report(report: &ContextBudgetReport) -> Self {
        Self {
            created_at_unix_ms: now_unix_ms(),
            strategy: report.strategy,
            budget_tokens: report.budget_tokens,
            reserve_tokens: report.reserve_tokens,
            effective_budget_tokens: report.effective_budget_tokens,
            pre_messages: report.pre_messages,
            post_messages: report.post_messages,
            dropped_messages: report.pre_messages.saturating_sub(report.post_messages),
            pre_tokens: report.pre_tokens,
            post_tokens: report.post_tokens,
            dropped_tokens: report.pre_tokens.saturating_sub(report.post_tokens),
            non_system: SessionContextBudgetClassSnapshot::from_stats(&report.non_system),
            regular_system: SessionContextBudgetClassSnapshot::from_stats(&report.regular_system),
            summary_system: SessionContextBudgetClassSnapshot::from_stats(&report.summary_system),
        }
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
        })
}

impl Agent {
    pub(crate) async fn record_context_budget_snapshot(
        &self,
        session_id: &str,
        report: &ContextBudgetReport,
    ) {
        let snapshot = SessionContextBudgetSnapshot::from_report(report);
        let mut guard = self.context_budget_snapshots.write().await;
        guard.insert(session_id.to_string(), snapshot);
    }

    /// Return latest context-budget snapshot for a session, if available.
    pub async fn inspect_context_budget_snapshot(
        &self,
        session_id: &str,
    ) -> Option<SessionContextBudgetSnapshot> {
        let guard = self.context_budget_snapshots.read().await;
        guard.get(session_id).copied()
    }
}
