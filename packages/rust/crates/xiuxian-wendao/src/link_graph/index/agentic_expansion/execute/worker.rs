use crate::link_graph::agentic::{
    LinkGraphAgenticExecutionConfig, LinkGraphAgenticWorkerExecution, LinkGraphAgenticWorkerPhase,
    LinkGraphAgenticWorkerPlan, LinkGraphSuggestedLinkRequest,
    suggested_link_signature_from_request, valkey_suggested_link_log,
};
use std::collections::HashSet;
use std::ops::ControlFlow;
use std::time::Instant;

struct WorkerPersistOutcome {
    persisted: bool,
    persist_phase_ms: f64,
    persist_attempts: usize,
    errors: Vec<String>,
}

fn initialize_worker_run(worker: &LinkGraphAgenticWorkerPlan) -> LinkGraphAgenticWorkerExecution {
    LinkGraphAgenticWorkerExecution {
        worker_id: worker.worker_id,
        pair_budget: worker.pair_count,
        processed_pairs: 0,
        prepared_proposals: 0,
        persisted_proposals: 0,
        skipped_duplicates: 0,
        failed_proposals: 0,
        persist_attempts: 0,
        phases: Vec::new(),
        timed_out: false,
        elapsed_ms: 0.0,
        estimated_prompt_tokens: 0,
        estimated_completion_tokens: 0,
    }
}

fn is_global_budget_exhausted(global_started: Instant, global_budget_ms: f64) -> bool {
    global_started.elapsed().as_secs_f64() * 1000.0 >= global_budget_ms
}

fn is_worker_budget_exhausted(worker_started: Instant, worker_budget_ms: f64) -> bool {
    worker_started.elapsed().as_secs_f64() * 1000.0 >= worker_budget_ms
}

fn build_suggested_link_request(
    query_text: Option<&str>,
    normalized: &LinkGraphAgenticExecutionConfig,
    worker_id: usize,
    left_id: &str,
    right_id: &str,
    priority: f64,
) -> LinkGraphSuggestedLinkRequest {
    let evidence = build_execution_evidence(
        &normalized.evidence_prefix,
        query_text,
        worker_id,
        left_id,
        right_id,
        priority,
    );
    LinkGraphSuggestedLinkRequest {
        source_id: left_id.to_string(),
        target_id: right_id.to_string(),
        relation: normalized.relation.clone(),
        confidence: priority.clamp(0.0, 1.0),
        evidence,
        agent_id: normalized.agent_id.clone(),
        created_at_unix: normalized.created_at_unix,
    }
}

fn persist_request_with_retries(
    request: &LinkGraphSuggestedLinkRequest,
    attempts: usize,
    worker_id: usize,
    left_id: &str,
    right_id: &str,
) -> WorkerPersistOutcome {
    let (persist_phase_ms, persist_attempts, errors, persisted) = match (0..attempts).try_fold(
        (0.0, 0usize, Vec::new()),
        |(persist_phase_ms, persist_attempts, mut errors), _| {
            let persist_started = Instant::now();
            let elapsed_ms = || persist_started.elapsed().as_secs_f64() * 1000.0;
            let next_attempts = persist_attempts.saturating_add(1);
            match valkey_suggested_link_log(request) {
                Ok(_) => ControlFlow::Break((
                    persist_phase_ms + elapsed_ms(),
                    next_attempts,
                    errors,
                    true,
                )),
                Err(err) => {
                    errors.push(format!(
                        "worker={worker_id} source={left_id} target={right_id} error={err}"
                    ));
                    ControlFlow::Continue((persist_phase_ms + elapsed_ms(), next_attempts, errors))
                }
            }
        },
    ) {
        ControlFlow::Break(outcome) => outcome,
        ControlFlow::Continue((persist_phase_ms, persist_attempts, errors)) => {
            (persist_phase_ms, persist_attempts, errors, false)
        }
    };
    WorkerPersistOutcome {
        persisted,
        persist_phase_ms,
        persist_attempts,
        errors,
    }
}

fn push_bounded_errors(errors: &mut Vec<String>, new_errors: Vec<String>) {
    for error in new_errors {
        if errors.len() >= 16 {
            break;
        }
        errors.push(error);
    }
}

fn finalize_worker_run(
    worker_run: &mut LinkGraphAgenticWorkerExecution,
    worker_started: Instant,
    prepare_phase_ms: f64,
    dedupe_phase_ms: f64,
    persist_phase_ms: f64,
) {
    worker_run.elapsed_ms = worker_started.elapsed().as_secs_f64() * 1000.0;
    worker_run.phases = vec![
        LinkGraphAgenticWorkerPhase {
            phase: "worker.prepare".to_string(),
            duration_ms: prepare_phase_ms,
            item_count: worker_run.prepared_proposals,
        },
        LinkGraphAgenticWorkerPhase {
            phase: "worker.dedupe".to_string(),
            duration_ms: dedupe_phase_ms,
            item_count: worker_run.processed_pairs,
        },
        LinkGraphAgenticWorkerPhase {
            phase: "worker.persist".to_string(),
            duration_ms: persist_phase_ms,
            item_count: worker_run.persist_attempts,
        },
        LinkGraphAgenticWorkerPhase {
            phase: "worker.total".to_string(),
            duration_ms: worker_run.elapsed_ms,
            item_count: worker_run.processed_pairs,
        },
    ];
}

pub(super) fn execute_worker(
    worker: &LinkGraphAgenticWorkerPlan,
    query_text: Option<&str>,
    normalized: &LinkGraphAgenticExecutionConfig,
    global_started: Instant,
    global_budget_ms: f64,
    idempotency_signatures: &mut HashSet<String>,
    errors: &mut Vec<String>,
) -> (LinkGraphAgenticWorkerExecution, bool) {
    let worker_started = Instant::now();
    let mut worker_run = initialize_worker_run(worker);
    let mut prepare_phase_ms = 0.0;
    let mut dedupe_phase_ms = 0.0;
    let mut persist_phase_ms = 0.0;
    let mut hit_global_budget = false;

    for pair in &worker.pairs {
        if is_global_budget_exhausted(global_started, global_budget_ms) {
            worker_run.timed_out = true;
            hit_global_budget = true;
            break;
        }
        if is_worker_budget_exhausted(worker_started, normalized.worker_time_budget_ms) {
            worker_run.timed_out = true;
            break;
        }

        worker_run.processed_pairs = worker_run.processed_pairs.saturating_add(1);
        let prepare_started = Instant::now();
        let request = build_suggested_link_request(
            query_text,
            normalized,
            worker.worker_id,
            pair.left_id.as_str(),
            pair.right_id.as_str(),
            pair.priority,
        );
        prepare_phase_ms += prepare_started.elapsed().as_secs_f64() * 1000.0;

        worker_run.prepared_proposals = worker_run.prepared_proposals.saturating_add(1);
        worker_run.estimated_prompt_tokens = worker_run
            .estimated_prompt_tokens
            .saturating_add(estimate_prompt_tokens(&request));

        if normalized.persist_suggestions {
            let dedupe_started = Instant::now();
            let signature = suggested_link_signature_from_request(&request);
            let is_duplicate = idempotency_signatures.contains(&signature);
            dedupe_phase_ms += dedupe_started.elapsed().as_secs_f64() * 1000.0;
            if is_duplicate {
                worker_run.skipped_duplicates = worker_run.skipped_duplicates.saturating_add(1);
            } else {
                let persist_outcome = persist_request_with_retries(
                    &request,
                    normalized.persist_retry_attempts,
                    worker.worker_id,
                    pair.left_id.as_str(),
                    pair.right_id.as_str(),
                );
                worker_run.persist_attempts = worker_run
                    .persist_attempts
                    .saturating_add(persist_outcome.persist_attempts);
                persist_phase_ms += persist_outcome.persist_phase_ms;
                push_bounded_errors(errors, persist_outcome.errors);

                if persist_outcome.persisted {
                    worker_run.persisted_proposals =
                        worker_run.persisted_proposals.saturating_add(1);
                    idempotency_signatures.insert(signature);
                } else {
                    worker_run.failed_proposals = worker_run.failed_proposals.saturating_add(1);
                }
            }
        }
    }

    finalize_worker_run(
        &mut worker_run,
        worker_started,
        prepare_phase_ms,
        dedupe_phase_ms,
        persist_phase_ms,
    );
    (worker_run, hit_global_budget)
}

fn build_execution_evidence(
    evidence_prefix: &str,
    query: Option<&str>,
    worker_id: usize,
    left_id: &str,
    right_id: &str,
    priority: f64,
) -> String {
    let mut out = format!(
        "{evidence_prefix}; worker={worker_id}; priority={priority:.4}; left={left_id}; right={right_id}"
    );
    if let Some(value) = query {
        out.push_str("; query=");
        out.push_str(value);
    }
    out
}

fn estimate_prompt_tokens(request: &LinkGraphSuggestedLinkRequest) -> u64 {
    let chars = request.source_id.len()
        + request.target_id.len()
        + request.relation.len()
        + request.evidence.len()
        + request.agent_id.len()
        + 96;
    // Approximate ceil(chars / 3.8) using integer math to avoid lossy float casts.
    let approx_tokens = chars.saturating_mul(10).saturating_add(37) / 38;
    u64::try_from(approx_tokens).unwrap_or(u64::MAX).max(1)
}
