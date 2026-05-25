//! JSON trace projection for the `SearchStrategyFlow` Flight service path.

use std::collections::HashSet;
use std::io::Cursor;
use std::path::Path;

use arrow::array::{Array, Float64Array, Int64Array, StringArray};
use arrow::ipc::reader::StreamReader;
use serde_json::{Value, json};
use xiuxian_wendao_runtime::transport::WENDAO_ARROW_FLIGHT_DATA_PLANE;

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

fn candidate_input_discovery_json(payload: &str) -> Result<Value, String> {
    if payload.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(payload)
        .map_err(|error| format!("parse SearchStrategyFlow candidate discovery receipt: {error}"))
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

fn validation_json(
    response: &SearchStrategyFlowServiceResponse,
    selected_candidate_ids: &HashSet<String>,
    query_understanding: &[Value],
    total_context: i64,
    selected_context: i64,
) -> Value {
    let required_evidence = required_evidence_values(query_understanding);
    let selected_required_evidence = selected_required_evidence_values(
        required_evidence.as_slice(),
        response.frontier.as_slice(),
    );
    let missing_required_evidence = required_evidence
        .iter()
        .filter(|evidence| !selected_required_evidence.contains(evidence))
        .cloned()
        .collect::<Vec<_>>();
    json!({
        "noVectorMode": response.candidates.iter().all(|row| row.semantic_score == 0.0),
        "materializedTopCandidate": response.planner_actions.iter().any(|row| {
            row.action_kind == "materialize" && selected_candidate_ids.contains(&row.candidate_id)
        }),
        "blockedEvidencePruned": response.candidates.iter().all(|row| {
            !row.blocked || !selected_candidate_ids.contains(&row.candidate_id)
        }),
        "selectedContextReduced": selected_context < total_context,
        "requiredEvidenceCovered": missing_required_evidence.is_empty(),
        "selectedRequiredEvidence": selected_required_evidence,
        "missingRequiredEvidence": missing_required_evidence,
    })
}

fn selected_candidate_ids(rows: &[SearchStrategyFlowFrontierRow]) -> HashSet<String> {
    rows.iter()
        .filter(|row| row.selected)
        .map(|row| row.candidate_id.clone())
        .collect()
}

fn required_evidence_values(query_understanding: &[Value]) -> Vec<String> {
    let mut values = query_understanding
        .iter()
        .filter_map(|row| {
            row.get("requiredEvidence")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
        })
        .map(str::to_owned)
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn selected_required_evidence_values(
    required_evidence: &[String],
    frontier: &[SearchStrategyFlowFrontierRow],
) -> Vec<String> {
    let selected_buckets = frontier
        .iter()
        .filter(|row| row.selected)
        .map(|row| frontier_route_bucket(row.candidate_id.as_str()))
        .collect::<HashSet<_>>();
    required_evidence
        .iter()
        .filter(|evidence| {
            selected_buckets.contains(required_evidence_route_bucket(evidence.as_str()))
        })
        .cloned()
        .collect()
}

fn required_evidence_route_bucket(evidence: &str) -> &str {
    match evidence {
        "ownership_boundary" => "authority",
        "validation_path" => "validation",
        "relation_path" => "link_graph",
        "page_index_seed" => "page_index",
        _ => evidence,
    }
}

pub(super) fn frontier_route_bucket(candidate_id: &str) -> &'static str {
    let normalized = candidate_id.to_ascii_lowercase();
    let (source_path, heading_anchor) = normalized
        .split_once('#')
        .map_or((normalized.as_str(), ""), |(path, anchor)| (path, anchor));

    if !frontier_source_is_code_or_test(source_path)
        && (frontier_candidate_is_authority(heading_anchor)
            || frontier_source_is_authority(source_path, normalized.as_str()))
    {
        return "authority";
    }
    if frontier_candidate_is_validation(heading_anchor) {
        return "validation";
    }
    if frontier_audit_document_is_validation(source_path, heading_anchor) {
        return "validation";
    }
    if let Some(route) = frontier_structural_source_route(source_path) {
        return route;
    }
    if let Some(route) = frontier_explicit_candidate_route(heading_anchor, source_path) {
        return route;
    }
    if frontier_markdown_authority_path(source_path, normalized.as_str()) {
        return "authority";
    }

    "general"
}

fn frontier_structural_source_route(source_path: &str) -> Option<&'static str> {
    if candidate_path_matches(
        source_path,
        &[
            "docs/30_search_strategy",
            "search_strategy_flow",
            "search-strategy-flow",
        ],
    ) {
        return Some("search_strategy");
    }
    if candidate_path_matches(
        source_path,
        &[
            "docs/20_page_index",
            "page_index",
            "pageindex",
            "reasoning_tree",
            "reasoning-tree",
        ],
    ) {
        return Some("page_index");
    }
    if candidate_path_matches(
        source_path,
        &[
            "docs/10_graph_compute",
            "link_graph",
            "link-graph",
            "linkgraph",
            "graph_compute",
            "relation",
        ],
    ) {
        return Some("link_graph");
    }
    if candidate_path_matches(
        source_path,
        &[
            "docs/90_validation",
            "docs/testing",
            "validation",
            "verify",
            "gate",
        ],
    ) {
        return Some("validation");
    }

    None
}

fn frontier_explicit_candidate_route(
    heading_anchor: &str,
    source_path: &str,
) -> Option<&'static str> {
    if frontier_candidate_is_link_graph(heading_anchor) {
        return Some("link_graph");
    }
    if frontier_candidate_is_validation(heading_anchor) {
        return Some("validation");
    }
    if frontier_candidate_is_page_index(heading_anchor) {
        return Some("page_index");
    }
    if frontier_candidate_is_search_strategy(heading_anchor) {
        return Some("search_strategy");
    }
    if candidate_path_matches(
        source_path,
        &[
            "packages/python/wendao-knowledge-retrieval-benchmark/docs/profile_contract.md",
            "packages/python/wendao-knowledge-retrieval-benchmark/docs/architecture.md",
        ],
    ) {
        return Some("validation");
    }
    if candidate_path_matches(
        source_path,
        &[
            "packages/rust/crates/xiuxian-wendao-attachments/readme.md",
            "packages/python/xiuxian-wendao-analyzer/readme.md",
        ],
    ) {
        return Some("link_graph");
    }

    None
}

fn frontier_candidate_is_authority(text: &str) -> bool {
    candidate_contains(
        text,
        &[
            "authority",
            "ownership",
            "owner-boundary",
            "owner-boundaries",
            "ownership-boundary",
            "ownership_boundary",
            "package-owner",
            "source-authority",
            "ssot",
            "single-source-of-truth",
        ],
    )
}

fn frontier_source_is_authority(source_path: &str, candidate_id: &str) -> bool {
    source_path == "docs/rfcs/2026-03-26-wendao-query-engine-rfc.md"
        || source_path == "packages/rust/crates/xiuxian-wendao-julia/readme.md"
        || source_path.starts_with("packages/rust/crates/xiuxian-wendao-julia/docs/")
        || source_path == "packages/rust/crates/xiuxian-wendao-studio/readme.md"
        || candidate_id.contains("current-ownership-matrix")
}

fn frontier_candidate_is_validation(text: &str) -> bool {
    candidate_contains(
        text,
        &[
            "validation",
            "validated",
            "validate",
            "verification",
            "verify",
            "verified",
            "gate",
            "package-test",
            "promotion-boundary",
            "proof",
            "test-proof",
            "evidence-calibration",
            "calibration",
            "audit",
            "benchmark",
            "baseline",
            "profile_contract",
            "profile-contract",
            "contract",
            "coverage",
            "quality",
            "fallback",
            "materialization",
            "closing-report",
        ],
    )
}

fn frontier_audit_document_is_validation(source_path: &str, heading_anchor: &str) -> bool {
    is_audit_report_markdown_path(source_path)
        && (heading_anchor.is_empty() || heading_anchor == "document")
}

fn is_audit_report_markdown_path(source_path: &str) -> bool {
    source_path.contains("-audit.")
        || source_path.contains("_audit.")
        || source_path.ends_with("-audit.md")
        || source_path.ends_with("_audit.md")
}

fn frontier_candidate_is_link_graph(text: &str) -> bool {
    candidate_contains(
        text,
        &[
            "link_graph",
            "link-graph",
            "linkgraph",
            "graph_compute",
            "graph-compute",
            "relation",
            "relationship",
            "ppr",
            "fanout",
            "placement",
            "belongs",
        ],
    )
}

fn frontier_candidate_is_page_index(text: &str) -> bool {
    candidate_contains(
        text,
        &[
            "page_index",
            "page-index",
            "pageindex",
            "reasoning_tree",
            "reasoning-tree",
            "document_projection",
            "document-projection",
            "projected-doc",
            "reading-order",
            "section-grounding",
        ],
    )
}

fn frontier_candidate_is_search_strategy(text: &str) -> bool {
    candidate_contains(
        text,
        &[
            "search_strategy",
            "search-strategy",
            "searchstrategyflow",
            "strategy-flow",
            "strategy flow",
        ],
    )
}

fn frontier_markdown_authority_path(source_path: &str, candidate_id: &str) -> bool {
    let path = std::path::Path::new(source_path);
    let is_markdown = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"));
    if !is_markdown {
        return false;
    }

    source_path == "agents.md"
        || source_path.starts_with("docs/rfcs/")
        || source_path.starts_with("docs/standards/")
        || (source_path.starts_with("docs/developer/")
            && frontier_candidate_is_authority(candidate_id))
        || (source_path.starts_with("packages/")
            && path
                .file_name()
                .is_some_and(|file_name| file_name.eq_ignore_ascii_case("readme.md")))
}

fn frontier_source_is_code_or_test(source_path: &str) -> bool {
    source_path.contains("/tests/")
        || source_path.contains("/test/")
        || source_path.starts_with("tests/")
        || [".rs", ".ts", ".tsx", ".js", ".jsx", ".jl", ".py"]
            .iter()
            .any(|extension| source_path.ends_with(extension))
}

fn candidate_contains(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn candidate_path_matches(source_path: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| source_path.contains(needle))
}

fn strategy_budget_json(query_understanding: &[Value]) -> Value {
    if query_understanding.is_empty() {
        return json!({
            "source": "default",
            "loopBudget": 1,
            "judgementBudget": 1,
            "beamWidth": 3,
        });
    }
    json!({
        "source": "query_understanding",
        "loopBudget": max_int_field(query_understanding, "recommendedLoopBudget", 1),
        "judgementBudget": max_int_field(query_understanding, "recommendedJudgementBudget", 1),
        "beamWidth": max_int_field(query_understanding, "recommendedBeamWidth", 3),
    })
}

fn max_int_field(rows: &[Value], field: &str, default: i64) -> i64 {
    rows.iter()
        .filter_map(|row| row.get(field).and_then(Value::as_i64))
        .max()
        .unwrap_or(default)
}

fn context_reduction_ratio(total_context: i64, selected_context: i64) -> f64 {
    if total_context <= 0 {
        0.0
    } else {
        1.0 - (non_negative_i64_to_f64(selected_context) / non_negative_i64_to_f64(total_context))
    }
}

fn non_negative_i64_to_f64(value: i64) -> f64 {
    let value = u32::try_from(value.max(0)).unwrap_or(u32::MAX);
    f64::from(value)
}

fn decode_query_understanding_trace_rows(payload: &[u8]) -> Result<Vec<Value>, String> {
    let batches = decode_arrow_ipc_batches(payload, "query_understanding")?;
    let mut rows = Vec::new();
    for batch in batches {
        for row_index in 0..batch.num_rows() {
            rows.push(json!({
                "flowId": string_value(&batch, "flow_id", row_index)?,
                "intentId": string_value(&batch, "intent_id", row_index)?,
                "signalId": string_value(&batch, "signal_id", row_index)?,
                "signalKind": string_value(&batch, "signal_kind", row_index)?,
                "signalValue": string_value(&batch, "signal_value", row_index)?,
                "confidence": float_value(&batch, "confidence", row_index)?,
                "routeHint": string_value(&batch, "route_hint", row_index)?,
                "requiredEvidence": string_value(&batch, "required_evidence", row_index)?,
                "ambiguity": float_value(&batch, "ambiguity", row_index)?,
                "weight": float_value(&batch, "weight", row_index)?,
                "recommendedLoopBudget": int_value(&batch, "recommended_loop_budget", row_index)?,
                "recommendedJudgementBudget": int_value(&batch, "recommended_judgement_budget", row_index)?,
                "recommendedBeamWidth": int_value(&batch, "recommended_beam_width", row_index)?,
                "reason": string_value(&batch, "reason", row_index)?,
            }));
        }
    }
    Ok(rows)
}

fn arrow_ipc_row_count(payload: &[u8]) -> Result<usize, String> {
    decode_arrow_ipc_batches(payload, "ontology_registry").map(|batches| {
        batches
            .iter()
            .map(arrow::record_batch::RecordBatch::num_rows)
            .sum()
    })
}

fn decode_arrow_ipc_batches(
    payload: &[u8],
    label: &str,
) -> Result<Vec<arrow::record_batch::RecordBatch>, String> {
    let reader = StreamReader::try_new(Cursor::new(payload), None)
        .map_err(|error| format!("decode SearchStrategyFlow {label} Arrow IPC: {error}"))?;
    reader
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("decode SearchStrategyFlow {label} Arrow IPC batch: {error}"))
}

fn string_value(
    batch: &arrow::record_batch::RecordBatch,
    column: &str,
    row_index: usize,
) -> Result<String, String> {
    let array = batch
        .column_by_name(column)
        .ok_or_else(|| format!("query_understanding missing `{column}`"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("query_understanding `{column}` must be Utf8"))?;
    if array.is_null(row_index) {
        return Ok(String::new());
    }
    Ok(array.value(row_index).to_owned())
}

fn float_value(
    batch: &arrow::record_batch::RecordBatch,
    column: &str,
    row_index: usize,
) -> Result<f64, String> {
    let array = batch
        .column_by_name(column)
        .ok_or_else(|| format!("query_understanding missing `{column}`"))?
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| format!("query_understanding `{column}` must be Float64"))?;
    if array.is_null(row_index) {
        return Ok(0.0);
    }
    Ok(array.value(row_index))
}

fn int_value(
    batch: &arrow::record_batch::RecordBatch,
    column: &str,
    row_index: usize,
) -> Result<i64, String> {
    let array = batch
        .column_by_name(column)
        .ok_or_else(|| format!("query_understanding missing `{column}`"))?
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| format!("query_understanding `{column}` must be Int64"))?;
    if array.is_null(row_index) {
        return Ok(0);
    }
    Ok(array.value(row_index))
}
