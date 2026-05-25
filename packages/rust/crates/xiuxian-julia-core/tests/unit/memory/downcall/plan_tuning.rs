use xiuxian_memory_engine::RecallPlanTuning;

use super::fetch_plan_tuning_advice_rows_from_inputs;
use crate::memory::host::MemoryPlanTuningInputs;
use crate::memory::test_support::{
    plan_tuning_response_batch, runtime_for_test, spawn_memory_service,
};

fn sample_inputs() -> Vec<MemoryPlanTuningInputs> {
    vec![MemoryPlanTuningInputs {
        scope: "repo".to_string(),
        scenario_pack: Some("searchinfra".to_string()),
        current_plan: RecallPlanTuning {
            k1: 8,
            k2: 4,
            lambda: 0.7,
            min_score: 0.18,
            max_context_chars: 960,
        },
        feedback_bias: 0.2,
        recent_success_rate: 0.4,
        recent_failure_rate: 0.3,
        recent_latency_ms: 250,
    }]
}

#[tokio::test]
async fn fetch_plan_tuning_advice_rows_from_inputs_roundtrips() {
    let route = "/memory/plan_tuning";
    let (base_url, server) = spawn_memory_service(plan_tuning_response_batch()).await;
    let runtime = runtime_for_test(base_url, route);

    let rows = fetch_plan_tuning_advice_rows_from_inputs(&runtime, &sample_inputs())
        .await
        .unwrap_or_else(|error| panic!("plan-tuning downcall should succeed: {error}"));

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].next_k1, 12);

    server.abort();
}
