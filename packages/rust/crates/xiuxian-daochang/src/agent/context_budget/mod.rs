//! Context budget state machine and memory compaction entrypoints.

mod classify;
mod prune;
mod selection;
mod truncate;
mod types;

pub use self::prune::prune_messages_for_token_budget;
pub(crate) use self::prune::prune_messages_for_token_budget_with_strategy;
pub(crate) use self::types::{
    ContextBudgetClassStats, ContextBudgetReport, SESSION_SUMMARY_MESSAGE_NAME,
};
