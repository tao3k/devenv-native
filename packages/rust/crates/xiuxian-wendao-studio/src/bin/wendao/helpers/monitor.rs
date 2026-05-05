//! Monitor-related CLI helpers.

use std::cmp::Ordering;

use serde_json::json;
use xiuxian_wendao::{LinkGraphAgenticExecutionResult, LinkGraphRelatedPprDiagnostics};

pub(crate) fn build_agentic_monitor_phases(
    result: &LinkGraphAgenticExecutionResult,
) -> Vec<serde_json::Value> {
    let mut phases: Vec<serde_json::Value> = vec![
        json!({
            "phase": "agentic.plan",
            "duration_ms": result.plan.elapsed_ms,
            "extra": {
                "candidate_notes": result.plan.candidate_notes,
                "selected_pairs": result.plan.selected_pairs,
                "timed_out": result.plan.timed_out,
            }
        }),
        json!({
            "phase": "agentic.execute.total",
            "duration_ms": result.elapsed_ms,
            "extra": {
                "worker_runs": result.worker_runs.len(),
                "prepared_proposals": result.prepared_proposals,
                "persisted_proposals": result.persisted_proposals,
                "timed_out": result.timed_out,
            }
        }),
    ];

    for worker_run in &result.worker_runs {
        for phase in &worker_run.phases {
            phases.push(json!({
                "phase": format!("agentic.{}", phase.phase),
                "duration_ms": phase.duration_ms,
                "extra": {
                    "worker_id": worker_run.worker_id,
                    "item_count": phase.item_count,
                    "processed_pairs": worker_run.processed_pairs,
                    "timed_out": worker_run.timed_out,
                }
            }));
        }
    }

    phases
}

pub(crate) fn build_agentic_monitor_summary(phases: &[serde_json::Value]) -> serde_json::Value {
    let slowest_phase = phases
        .iter()
        .max_by(|left, right| {
            phase_duration_ms(left)
                .partial_cmp(&phase_duration_ms(right))
                .unwrap_or(Ordering::Equal)
        })
        .cloned();

    json!({
        "phase_count": phases.len(),
        "slowest_phase": slowest_phase,
    })
}

pub(crate) fn build_related_monitor_phases(
    diagnostics: Option<LinkGraphRelatedPprDiagnostics>,
) -> Vec<serde_json::Value> {
    let Some(row) = diagnostics else {
        return Vec::new();
    };
    let mut phases: Vec<serde_json::Value> = Vec::new();
    phases.push(json!({
        "phase": "link_graph.related.ppr",
        "duration_ms": row.total_duration_ms,
        "extra": {
            "candidate_count": row.candidate_count,
            "candidate_cap": row.candidate_cap,
            "candidate_capped": row.candidate_capped,
            "graph_node_count": row.graph_node_count,
            "subgraph_count": row.subgraph_count,
            "subgraph_mode": row.subgraph_mode,
            "horizon_restricted": row.horizon_restricted,
            "time_budget_ms": row.time_budget_ms,
            "timed_out": row.timed_out,
            "iteration_count": row.iteration_count,
            "final_residual": row.final_residual,
        }
    }));
    phases.push(json!({
        "phase": "link_graph.related.subgraph.partition",
        "duration_ms": row.partition_duration_ms,
        "extra": {
            "subgraph_count": row.subgraph_count,
            "partition_max_node_count": row.partition_max_node_count,
            "partition_min_node_count": row.partition_min_node_count,
            "partition_avg_node_count": row.partition_avg_node_count,
            "timed_out": row.timed_out,
        }
    }));
    phases.push(json!({
        "phase": "link_graph.related.subgraph.fusion",
        "duration_ms": row.fusion_duration_ms,
        "extra": {
            "subgraph_count": row.subgraph_count,
            "timed_out": row.timed_out,
        }
    }));
    phases
}

fn phase_duration_ms(phase: &serde_json::Value) -> f64 {
    phase
        .get("duration_ms")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0)
}
