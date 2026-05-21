//! Command dispatch implementation for `wendao` CLI.
//!
//! This module dispatches CLI commands to their respective handler modules.
//!
//! Each handler module implements the logic for a specific command.

#[path = "agentic/mod.rs"]
mod agentic;
#[path = "attachments.rs"]
mod attachments;
#[path = "audit.rs"]
mod audit;
mod dispatch;
#[path = "docs.rs"]
mod docs;
#[path = "episteme.rs"]
mod episteme;
#[path = "fix.rs"]
mod fix;
#[cfg(feature = "zhenfa-router")]
#[path = "gateway/mod.rs"]
mod gateway;
#[path = "graph/mod.rs"]
mod graph;
#[path = "hmas.rs"]
mod hmas;
#[cfg(feature = "zhenfa-router")]
#[path = "query/mod.rs"]
mod query;
#[path = "repo.rs"]
mod repo;
#[path = "saliency.rs"]
mod saliency;
#[path = "search.rs"]
mod search;
#[path = "sentinel.rs"]
mod sentinel;

pub(crate) use dispatch::{can_execute_immediate, execute, execute_immediate};

#[cfg(test)]
use dispatch::client_context_from_cli;

#[cfg(test)]
#[path = "../../../../tests/unit/bin/wendao/execute.rs"]
mod tests;
