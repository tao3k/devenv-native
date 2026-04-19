use crate::helpers::{build_agentic_monitor_phases, build_agentic_monitor_summary, emit};
use crate::types::Cli;
use anyhow::{Context, Result};
use serde_json::json;
use xiuxian_wendao::{LinkGraphAgenticExecutionConfig, LinkGraphIndex};

pub(super) fn handle_plan(
    cli: &Cli,
    index: &LinkGraphIndex,
    query: Option<&str>,
    max_workers: Option<usize>,
    max_candidates: Option<usize>,
    max_pairs_per_worker: Option<usize>,
    time_budget_ms: Option<f64>,
) -> Result<()> {
    let mut config = LinkGraphIndex::resolve_agentic_expansion_config();
    if let Some(value) = max_workers {
        config.max_workers = value.max(1);
    }
    if let Some(value) = max_candidates {
        config.max_candidates = value.max(1);
    }
    if let Some(value) = max_pairs_per_worker {
        config.max_pairs_per_worker = value.max(1);
    }
    if let Some(value) = time_budget_ms {
        config.time_budget_ms = if value.is_finite() && value > 0.0 {
            value
        } else {
            config.time_budget_ms
        };
    }
    let plan = index.agentic_expansion_plan_with_config(query, config);
    emit(&plan, cli.output_or_json())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_run(
    cli: &Cli,
    index: &LinkGraphIndex,
    query: Option<&str>,
    max_workers: Option<usize>,
    max_candidates: Option<usize>,
    max_pairs_per_worker: Option<usize>,
    time_budget_ms: Option<f64>,
    worker_time_budget_ms: Option<f64>,
    persist: Option<bool>,
    persist_retry_attempts: Option<usize>,
    idempotency_scan_limit: Option<usize>,
    relation: Option<String>,
    agent_id: Option<String>,
    evidence_prefix: Option<String>,
    created_at_unix: Option<f64>,
    verbose: bool,
) -> Result<()> {
    let mut config: LinkGraphAgenticExecutionConfig =
        LinkGraphIndex::resolve_agentic_execution_config();
    if let Some(value) = max_workers {
        config.expansion.max_workers = value.max(1);
    }
    if let Some(value) = max_candidates {
        config.expansion.max_candidates = value.max(1);
    }
    if let Some(value) = max_pairs_per_worker {
        config.expansion.max_pairs_per_worker = value.max(1);
    }
    if let Some(value) = time_budget_ms {
        config.expansion.time_budget_ms = if value.is_finite() && value > 0.0 {
            value
        } else {
            config.expansion.time_budget_ms
        };
    }
    if let Some(value) = worker_time_budget_ms {
        config.worker_time_budget_ms = if value.is_finite() && value > 0.0 {
            value
        } else {
            config.worker_time_budget_ms
        };
    }
    if let Some(value) = persist {
        config.persist_suggestions = value;
    }
    if let Some(value) = persist_retry_attempts {
        config.persist_retry_attempts = value.max(1);
    }
    if let Some(value) = idempotency_scan_limit {
        config.idempotency_scan_limit = value.max(1);
    }
    if let Some(value) = relation {
        config.relation = value;
    }
    if let Some(value) = agent_id {
        config.agent_id = value;
    }
    if let Some(value) = evidence_prefix {
        config.evidence_prefix = value;
    }
    config.created_at_unix = created_at_unix;
    let result = index.agentic_expansion_execute_with_config(query, config);

    if verbose {
        let phases = build_agentic_monitor_phases(&result);
        let monitor = json!({
            "overview": {
                "elapsed_ms": result.elapsed_ms,
                "worker_runs": result.worker_runs.len(),
                "prepared_proposals": result.prepared_proposals,
                "persisted_proposals": result.persisted_proposals,
                "skipped_duplicates": result.skipped_duplicates,
                "failed_proposals": result.failed_proposals,
                "persist_attempts": result.persist_attempts,
                "timed_out": result.timed_out,
            },
            "bottlenecks": build_agentic_monitor_summary(&phases),
        });
        let mut payload = serde_json::to_value(&result)
            .context("failed to serialize agentic execution result")?;
        if let Some(map) = payload.as_object_mut() {
            map.insert("phases".to_string(), json!(phases));
            map.insert("monitor".to_string(), monitor);
        }
        emit(&payload, cli.output_or_json())
    } else {
        emit(&result, cli.output_or_json())
    }
}
