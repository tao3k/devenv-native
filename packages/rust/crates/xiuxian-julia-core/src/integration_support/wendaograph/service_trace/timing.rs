use serde_json::{Value, json};

use crate::integration_support::search_strategy_flow_flight::SearchStrategyFlowServiceResponse;

/// Runtime timing measurements attached to a `SearchStrategyFlow` trace.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct SearchStrategyFlowTimingMeasurements {
    /// Rust-side candidate discovery and optional ontology-registry payload time.
    pub(crate) candidate_discovery: Option<f64>,
    /// Rust-to-Julia `SearchStrategyFlow` service roundtrip time.
    pub(crate) algorithm_service: Option<f64>,
    /// Rust-side post-service materialization enrichment time.
    pub(crate) materialization: Option<f64>,
}

pub(crate) fn search_strategy_flow_timing_breakdown_json(
    response: &SearchStrategyFlowServiceResponse,
    timing: SearchStrategyFlowTimingMeasurements,
) -> Value {
    let llm_judgement_required_count = response
        .planner_actions
        .iter()
        .filter(|row| row.requires_llm_judgement)
        .count();
    json!({
        "schemaVersion": "xiuxian_wendao.graph.search_strategy_flow.timing_breakdown.v1",
        "measured": timing.has_any_measurement(),
        "reportedBy": "rust-bridge-trace-contract",
        "coldStartMs": null,
        "warmSubmitMs": null,
        "materializationMs": timing.materialization,
        "llmJudgeMs": null,
        "algorithmServiceMs": timing.algorithm_service,
        "candidateDiscoveryMs": timing.candidate_discovery,
        "llmJudgementRequiredCount": llm_judgement_required_count,
        "materializationMeasured": timing.materialization.is_some(),
        "llmJudgeMeasured": false,
        "notes": [
            "timing slots are reserved for benchmark runners",
            "null values mean this trace path did not measure the segment",
        ],
    })
}

impl SearchStrategyFlowTimingMeasurements {
    fn has_any_measurement(self) -> bool {
        self.candidate_discovery.is_some()
            || self.algorithm_service.is_some()
            || self.materialization.is_some()
    }
}

pub(crate) fn search_strategy_flow_trace_with_materialization_timing(
    trace: &str,
    materialization_ms: f64,
) -> Result<String, String> {
    let mut value = serde_json::from_str::<Value>(trace)
        .map_err(|error| format!("parse SearchStrategyFlow trace for timing update: {error}"))?;
    let timing = value
        .get_mut("timingBreakdown")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| "SearchStrategyFlow trace missing timingBreakdown object".to_owned())?;
    timing.insert("measured".to_owned(), Value::Bool(true));
    timing.insert("materializationMeasured".to_owned(), Value::Bool(true));
    timing.insert(
        "materializationMs".to_owned(),
        finite_ms_value(materialization_ms, "materializationMs")?,
    );
    serde_json::to_string(&value)
        .map(|trace| format!("{trace}\n"))
        .map_err(|error| format!("serialize SearchStrategyFlow trace timing update: {error}"))
}

fn finite_ms_value(value: f64, field: &str) -> Result<Value, String> {
    if !value.is_finite() || value < 0.0 {
        return Err(format!("{field} must be a finite non-negative value"));
    }
    serde_json::Number::from_f64(value)
        .map(Value::Number)
        .ok_or_else(|| format!("serialize {field} as JSON number"))
}
