use crate::link_graph::agentic::{
    LinkGraphAgenticExecutionConfig, LinkGraphAgenticExecutionResult,
    LinkGraphAgenticWorkerExecution, suggested_link_signature_from_row,
    valkey_suggested_link_recent_latest,
};
use crate::link_graph::index::LinkGraphIndex;
use std::collections::HashSet;
use std::time::Instant;

use super::worker::execute_worker;

pub(in crate::link_graph::index::agentic_expansion) fn agentic_expansion_execute_with_config(
    index: &LinkGraphIndex,
    query: Option<&str>,
    config: LinkGraphAgenticExecutionConfig,
) -> LinkGraphAgenticExecutionResult {
    let normalized = config.normalized();
    let started = Instant::now();
    let plan = index.agentic_expansion_plan_with_config(query, normalized.expansion);
    let global_budget_ms = normalized.expansion.time_budget_ms.max(1.0);
    let query_text = plan.query.as_deref();

    let mut worker_runs: Vec<LinkGraphAgenticWorkerExecution> = Vec::new();
    let mut prepared_proposals = 0usize;
    let mut persisted_proposals = 0usize;
    let mut skipped_duplicates = 0usize;
    let mut failed_proposals = 0usize;
    let mut persist_attempts = 0usize;
    let mut timed_out = plan.timed_out;
    let mut errors: Vec<String> = Vec::new();
    let mut idempotency_signatures: HashSet<String> = HashSet::new();

    if normalized.persist_suggestions {
        match valkey_suggested_link_recent_latest(normalized.idempotency_scan_limit, None) {
            Ok(rows) => {
                for row in rows {
                    idempotency_signatures.insert(suggested_link_signature_from_row(&row));
                }
            }
            Err(err) => {
                if errors.len() < 16 {
                    errors.push(format!("idempotency preload failed: {err}"));
                }
            }
        }
    }

    for worker in &plan.workers {
        if started.elapsed().as_secs_f64() * 1000.0 >= global_budget_ms {
            timed_out = true;
            break;
        }

        let (worker_run, worker_hit_global_budget) = execute_worker(
            worker,
            query_text,
            &normalized,
            started,
            global_budget_ms,
            &mut idempotency_signatures,
            &mut errors,
        );

        prepared_proposals = prepared_proposals.saturating_add(worker_run.prepared_proposals);
        persisted_proposals = persisted_proposals.saturating_add(worker_run.persisted_proposals);
        skipped_duplicates = skipped_duplicates.saturating_add(worker_run.skipped_duplicates);
        failed_proposals = failed_proposals.saturating_add(worker_run.failed_proposals);
        persist_attempts = persist_attempts.saturating_add(worker_run.persist_attempts);
        if worker_hit_global_budget {
            timed_out = true;
        }
        worker_runs.push(worker_run);
        if worker_hit_global_budget {
            break;
        }
    }

    LinkGraphAgenticExecutionResult {
        query: plan.query.clone(),
        config: normalized,
        plan,
        worker_runs,
        prepared_proposals,
        persisted_proposals,
        skipped_duplicates,
        failed_proposals,
        persist_attempts,
        timed_out,
        elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
        errors,
    }
}
