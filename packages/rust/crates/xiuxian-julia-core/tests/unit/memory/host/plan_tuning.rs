use xiuxian_memory_engine::RecallPlanTuning;

use crate::memory::host::plan_tuning::{
    MemoryPlanTuningInputs, build_memory_plan_tuning_request_batch_from_inputs,
    build_memory_plan_tuning_request_rows_from_inputs,
};

fn sample_inputs() -> MemoryPlanTuningInputs {
    MemoryPlanTuningInputs {
        scope: "repo".to_string(),
        scenario_pack: Some("searchinfra".to_string()),
        current_plan: RecallPlanTuning {
            k1: 8,
            k2: 4,
            lambda: 0.7,
            min_score: 0.18,
            max_context_chars: 960,
        },
        feedback_bias: -0.4,
        recent_success_rate: 0.35,
        recent_failure_rate: 0.45,
        recent_latency_ms: 210,
    }
}

#[test]
fn build_memory_plan_tuning_request_rows_from_inputs_maps_host_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let rows = build_memory_plan_tuning_request_rows_from_inputs(&[sample_inputs()])?;

    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.scope, "repo");
    assert_eq!(row.scenario_pack.as_deref(), Some("searchinfra"));
    assert_eq!(row.current_k1, 8);
    assert_eq!(row.current_k2, 4);
    assert!((row.current_lambda - 0.7).abs() < 1e-6);
    assert!((row.current_min_score - 0.18).abs() < 1e-6);
    assert_eq!(row.current_max_context_chars, 960);
    assert!((row.feedback_bias + 0.4).abs() < 1e-6);
    assert!((row.recent_success_rate - 0.35).abs() < 1e-6);
    assert!((row.recent_failure_rate - 0.45).abs() < 1e-6);
    assert_eq!(row.recent_latency_ms, 210);

    Ok(())
}

#[test]
fn build_memory_plan_tuning_request_batch_from_inputs_materializes_staged_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let mut second = sample_inputs();
    second.scope = "workspace".to_string();
    second.scenario_pack = None;
    second.current_plan.k1 = 10;
    second.current_plan.k2 = 5;
    second.recent_latency_ms = 320;

    let batch = build_memory_plan_tuning_request_batch_from_inputs(&[sample_inputs(), second])?;

    assert_eq!(batch.num_rows(), 2);
    assert_eq!(batch.schema().fields().len(), 11);
    assert!(batch.column_by_name("scope").is_some());
    assert!(batch.column_by_name("current_lambda").is_some());
    assert!(batch.column_by_name("feedback_bias").is_some());

    Ok(())
}

#[test]
fn build_memory_plan_tuning_request_batch_from_inputs_rejects_invalid_shape() {
    let mut inputs = sample_inputs();
    inputs.current_plan.k2 = 9;

    let Err(error) = build_memory_plan_tuning_request_batch_from_inputs(&[inputs]) else {
        panic!("current_k2 greater than current_k1 must fail");
    };

    assert!(error.to_string().contains("current_k2"));
}

#[test]
fn build_memory_plan_tuning_request_rows_from_inputs_normalizes_feedback_bias()
-> Result<(), Box<dyn std::error::Error>> {
    let mut inputs = sample_inputs();
    inputs.feedback_bias = 1.6;

    let rows = build_memory_plan_tuning_request_rows_from_inputs(&[inputs])?;

    assert_eq!(rows.len(), 1);
    assert!((rows[0].feedback_bias - 1.0).abs() < 1e-6);

    Ok(())
}
