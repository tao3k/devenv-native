use crate::{LinkGraphIndex, LinkGraphSuggestedLinkRequest};
use anyhow::{Context, Result};

use super::plan_run::{handle_plan, handle_run};
use super::suggested_links::{handle_decide, handle_decisions, handle_log, handle_recent};
use crate::bin_support::wendao::types::{AgenticCommand, Cli, Command};

pub(in crate::bin_support::wendao::execute) fn handle(
    cli: &Cli,
    index: Option<&LinkGraphIndex>,
) -> Result<()> {
    let Command::Agentic { command } = &cli.command else {
        unreachable!("agentic handler must be called with agentic command");
    };

    match command {
        AgenticCommand::Log {
            source_id,
            target_id,
            relation,
            confidence,
            evidence,
            agent_id,
            created_at_unix,
        } => {
            let request = LinkGraphSuggestedLinkRequest {
                source_id: source_id.clone(),
                target_id: target_id.clone(),
                relation: relation.clone(),
                confidence: *confidence,
                evidence: evidence.clone(),
                agent_id: agent_id.clone(),
                created_at_unix: *created_at_unix,
            };
            handle_log(cli, &request)
        }
        AgenticCommand::Recent {
            limit,
            latest,
            state,
        } => handle_recent(cli, (*limit).max(1), *latest, state.map(Into::into)),
        AgenticCommand::Decide {
            suggestion_id,
            target_state,
            decided_by,
            reason,
            decided_at_unix,
        } => handle_decide(
            cli,
            suggestion_id,
            (*target_state).into(),
            decided_by,
            reason,
            *decided_at_unix,
        ),
        AgenticCommand::Decisions { limit } => handle_decisions(cli, (*limit).max(1)),
        AgenticCommand::Plan {
            query,
            max_workers,
            max_candidates,
            max_pairs_per_worker,
            time_budget_ms,
        } => {
            let index = index.context("link_graph index is required for agentic plan command")?;
            handle_plan(
                cli,
                index,
                query.as_deref(),
                *max_workers,
                *max_candidates,
                *max_pairs_per_worker,
                *time_budget_ms,
            )
        }
        AgenticCommand::Run {
            query,
            max_workers,
            max_candidates,
            max_pairs_per_worker,
            time_budget_ms,
            worker_time_budget_ms,
            persist,
            persist_retry_attempts,
            idempotency_scan_limit,
            relation,
            agent_id,
            evidence_prefix,
            created_at_unix,
            verbose,
        } => {
            let index = index.context("link_graph index is required for agentic run command")?;
            handle_run(
                cli,
                index,
                query.as_deref(),
                *max_workers,
                *max_candidates,
                *max_pairs_per_worker,
                *time_budget_ms,
                *worker_time_budget_ms,
                *persist,
                *persist_retry_attempts,
                *idempotency_scan_limit,
                relation.clone(),
                agent_id.clone(),
                evidence_prefix.clone(),
                *created_at_unix,
                *verbose,
            )
        }
    }
}
