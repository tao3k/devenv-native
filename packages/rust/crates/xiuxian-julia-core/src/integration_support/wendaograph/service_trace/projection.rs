use std::path::Path;

use serde_json::{Value, json};
use xiuxian_wendao_runtime::transport::WENDAO_ARROW_FLIGHT_DATA_PLANE;

use super::budget::{context_reduction_ratio, strategy_budget_json};
use super::decode::{
    arrow_ipc_row_count, candidate_input_discovery_json, decode_query_understanding_trace_rows,
};
use super::policy::search_strategy_flow_performance_policy_json;
use super::timing::{
    SearchStrategyFlowTimingMeasurements, search_strategy_flow_timing_breakdown_json,
};
use super::validation::{selected_candidate_ids, validation_json};
use crate::integration_support::search_strategy_flow_candidates::SearchStrategyFlowCandidateInputBatch;
use crate::integration_support::search_strategy_flow_flight::{
    SearchStrategyFlowFrontierRow, SearchStrategyFlowServiceCandidateRow,
    SearchStrategyFlowServicePlannerActionRow, SearchStrategyFlowServiceResponse,
};

/// Request metadata required to project a service response into a bridge trace.
pub(crate) struct SearchStrategyFlowServiceTraceRequest<'a> {
    pub(crate) intent: &'a str,
    pub(crate) search_root: &'a Path,
    pub(crate) candidate_batch: &'a SearchStrategyFlowCandidateInputBatch,
    pub(crate) service_base_url: &'a str,
    pub(crate) service_flight_route: &'a str,
    pub(crate) service_timeout_seconds: u64,
    pub(crate) response: &'a SearchStrategyFlowServiceResponse,
    pub(crate) query_understanding_payload: Option<&'a [u8]>,
    pub(crate) ontology_registry_payload: Option<&'a [u8]>,
    pub(crate) timing: SearchStrategyFlowTimingMeasurements,
}

/// Build the benchmark-compatible JSON trace for a Flight service response.
///
/// # Errors
///
/// Returns an error when the candidate discovery receipt or side-table Arrow
/// IPC payload cannot be decoded as trace metadata.
pub(crate) fn search_strategy_flow_service_trace_json(
    request: &SearchStrategyFlowServiceTraceRequest<'_>,
) -> Result<String, String> {
    let query_understanding = request
        .query_understanding_payload
        .map(decode_query_understanding_trace_rows)
        .transpose()?
        .unwrap_or_default();
    let ontology_registry_count = request
        .ontology_registry_payload
        .map(arrow_ipc_row_count)
        .transpose()?
        .unwrap_or_default();
    let selected_candidate_ids = selected_candidate_ids(request.response.frontier.as_slice());
    let total_context = request
        .response
        .candidates
        .iter()
        .map(|row| row.context_cost.max(0))
        .sum::<i64>();
    let selected_context = request
        .response
        .frontier
        .iter()
        .map(|row| row.context_budget.max(0))
        .sum::<i64>();
    let validation = validation_json(
        request.response,
        &selected_candidate_ids,
        query_understanding.as_slice(),
        total_context,
        selected_context,
    );
    let stage_receipts = stage_receipts_json(
        request.response,
        query_understanding.len(),
        total_context,
        selected_context,
        selected_candidate_ids.len(),
    );
    let trace = json!({
        "intent": request.intent,
        "backend": "rust-wendao-julia",
        "controlPlane": "rust",
        "strategyFlowDataPlane": WENDAO_ARROW_FLIGHT_DATA_PLANE,
        "strategyFlowService": {
            "dataPlane": WENDAO_ARROW_FLIGHT_DATA_PLANE,
            "baseUrl": request.service_base_url,
            "flightRoute": request.service_flight_route,
            "timeoutSeconds": request.service_timeout_seconds,
        },
        "performancePolicy": search_strategy_flow_performance_policy_json(),
        "timingBreakdown": search_strategy_flow_timing_breakdown_json(
            request.response,
            request.timing,
        ),
        "juliaProject": "",
        "graphProject": "",
        "searchRoot": request.search_root.display().to_string(),
        "candidateInputSource": request.candidate_batch.source,
        "candidateInputCount": request.candidate_batch.row_count,
        "candidateInputDiscovery": candidate_input_discovery_json(
            request.candidate_batch.discovery_receipt_json.as_str(),
        )?,
        "ontologyRegistryInputCount": ontology_registry_count,
        "queryUnderstanding": query_understanding,
        "strategyBudget": strategy_budget_json(query_understanding.as_slice()),
        "stageReceipts": stage_receipts,
        "candidates": candidates_json(request.response.candidates.as_slice()),
        "frontier": frontier_json(request.response.frontier.as_slice()),
        "plannerActions": planner_actions_json(request.response.planner_actions.as_slice()),
        "summary": {
            "candidateCount": request.response.candidates.len(),
            "selectedCount": selected_candidate_ids.len(),
            "plannerActionCount": request.response.planner_actions.len(),
            "totalContextCost": total_context,
            "selectedContextCost": selected_context,
            "contextReductionRatio": context_reduction_ratio(total_context, selected_context),
        },
        "validation": validation,
    });
    serde_json::to_string(&trace)
        .map(|trace| format!("{trace}\n"))
        .map_err(|error| format!("serialize SearchStrategyFlow service trace: {error}"))
}

fn candidates_json(rows: &[SearchStrategyFlowServiceCandidateRow]) -> Vec<Value> {
    rows.iter()
        .map(|row| {
            json!({
                "candidateId": row.candidate_id,
                "action": row.action,
                "reason": row.reason,
                "finalScore": row.final_score,
                "evidenceCoverage": row.evidence_coverage,
                "graphScore": row.graph_score,
                "authorityScore": row.authority_score,
                "semanticScore": row.semantic_score,
                "structuralScore": row.structural_score,
                "contextCost": row.context_cost,
                "blocked": row.blocked,
            })
        })
        .collect()
}

fn frontier_json(rows: &[SearchStrategyFlowFrontierRow]) -> Vec<Value> {
    rows.iter()
        .map(|row| {
            json!({
                "candidateId": row.candidate_id,
                "rank": row.rank,
                "selected": row.selected,
                "finalScore": row.final_score,
                "action": row.action,
                "contextBudget": row.context_budget,
                "judgementKind": row.judgement_kind,
            })
        })
        .collect()
}

fn planner_actions_json(rows: &[SearchStrategyFlowServicePlannerActionRow]) -> Vec<Value> {
    rows.iter()
        .map(|row| {
            json!({
                "actionKind": row.action_kind,
                "candidateId": row.candidate_id,
                "targetCandidateId": row.target_candidate_id,
                "cycleAllowed": row.cycle_allowed,
                "requiresLlmJudgement": row.requires_llm_judgement,
                "score": row.score,
                "contextBudget": row.context_budget,
                "reason": row.reason,
            })
        })
        .collect()
}

fn stage_receipts_json(
    response: &SearchStrategyFlowServiceResponse,
    query_understanding_count: usize,
    total_context: i64,
    selected_context: i64,
    selected_count: usize,
) -> Vec<Value> {
    let llm_action_count = response
        .planner_actions
        .iter()
        .filter(|row| row.requires_llm_judgement)
        .count();
    let cycle_action_count = response
        .planner_actions
        .iter()
        .filter(|row| row.cycle_allowed)
        .count();
    vec![
        json!({
            "stage": "query_understanding",
            "notebook": "notebooks/search_strategy_flow_query_understanding.jl",
            "inputCount": 1,
            "outputCount": query_understanding_count,
            "selectedCount": 0,
            "llmJudgementCount": 0,
            "cycleAllowedCount": 0,
            "contextBudget": 0,
            "summary": "intent to graph route hints, required evidence, ambiguity, and strategy budget",
        }),
        json!({
            "stage": "candidate_scoring",
            "notebook": "notebooks/search_strategy_flow_candidate_scoring.jl",
            "inputCount": response.candidates.len(),
            "outputCount": response.candidates.len(),
            "selectedCount": response.candidates.iter().filter(|row| row.action != "prune").count(),
            "llmJudgementCount": 0,
            "cycleAllowedCount": 0,
            "contextBudget": total_context,
            "summary": "graph evidence rows to deterministic score rows and branch actions",
        }),
        json!({
            "stage": "transition_inference",
            "notebook": "notebooks/search_strategy_flow_transition_inference.jl",
            "inputCount": response.candidates.len(),
            "outputCount": response.transition_count,
            "selectedCount": response.transition_count,
            "llmJudgementCount": 0,
            "cycleAllowedCount": 0,
            "contextBudget": 0,
            "summary": "score rows to revision transition kinds and missing-signal diagnostics",
        }),
        json!({
            "stage": "frontier_selection",
            "notebook": "notebooks/search_strategy_flow_frontier_selection.jl",
            "inputCount": response.candidates.len(),
            "outputCount": response.frontier.len(),
            "selectedCount": selected_count,
            "llmJudgementCount": response.frontier.iter().filter(|row| row.selected && row.judgement_kind == "subagent_branch_judgement").count(),
            "cycleAllowedCount": 0,
            "contextBudget": selected_context,
            "summary": "beam and context-budget bounded Agent-visible frontier",
        }),
        json!({
            "stage": "planner_actions",
            "notebook": "notebooks/search_strategy_flow_planner_actions.jl",
            "inputCount": response.frontier.len(),
            "outputCount": response.planner_actions.len(),
            "selectedCount": response.planner_actions.iter().filter(|row| row.action_kind != "stop").count(),
            "llmJudgementCount": llm_action_count,
            "cycleAllowedCount": cycle_action_count,
            "contextBudget": response.planner_actions.iter().map(|row| row.context_budget.max(0)).sum::<i64>(),
            "summary": "frontier and transition facts to materialize, refine, judge, compare, and stop actions",
        }),
    ]
}
