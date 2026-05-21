mod support;

use self::support::{
    calibration_response_batch, episodic_recall_response_batch, gate_score_response_batch,
    plan_tuning_response_batch, spawn_memory_service, write_memory_runtime_override,
};
use crate::link_graph::set_link_graph_wendao_config_override;
use crate::memory::julia::{
    ComputeClient, EpisodicRecallQueryInputs, MemoryCalibrationInputs, MemoryGateScoreEvidenceRow,
    MemoryJuliaCalibrationArtifactRow, MemoryJuliaEpisodicRecallScoreRow,
    MemoryJuliaGateScoreRecommendationRow, MemoryJuliaPlanTuningAdviceRow, MemoryLifecycleState,
    MemoryPlanTuningInputs, MemoryProjectionRow, MemoryUtilityLedger, RecallPlanTuning,
    resolve_memory_julia_compute_bindings, resolve_memory_julia_compute_runtime,
};
use serial_test::serial;
use std::fs;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

#[test]
fn julia_memory_bridge_exports_only_thin_host_surfaces() {
    let _ = ComputeClient::configured;
    let _ = resolve_memory_julia_compute_runtime;
    let _ = resolve_memory_julia_compute_bindings;

    let _ = std::mem::size_of::<ComputeClient>();
    let _ = std::mem::size_of::<EpisodicRecallQueryInputs>();
    let _ = std::mem::size_of::<MemoryProjectionRow>();
    let _ = std::mem::size_of::<MemoryGateScoreEvidenceRow>();
    let _ = std::mem::size_of::<MemoryPlanTuningInputs>();
    let _ = std::mem::size_of::<MemoryCalibrationInputs>();
    let _ = std::mem::size_of::<MemoryJuliaEpisodicRecallScoreRow>();
    let _ = std::mem::size_of::<MemoryJuliaGateScoreRecommendationRow>();
    let _ = std::mem::size_of::<MemoryJuliaPlanTuningAdviceRow>();
    let _ = std::mem::size_of::<MemoryJuliaCalibrationArtifactRow>();
}

#[test]
#[serial]
fn resolve_memory_julia_compute_runtime_reads_override_values()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        r#"[memory.julia_compute]
enabled = true
base_url = "http://127.0.0.1:18825"
schema_version = "v1"
plugin_id = "wendao.memory"
health_route = "/healthz"
service_mode = "stream"
scenario_pack = "searchinfra"
timeout_secs = 3
fallback_mode = "rust"
shadow_compare = true

[memory.julia_compute.routes]
episodic_recall = "/memory/episodic_recall"
memory_gate_score = "/memory/gate_score"
memory_plan_tuning = "/memory/plan_tuning"
memory_calibration = "/memory/calibration"
"#,
    )?;
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());

    let runtime = resolve_memory_julia_compute_runtime();
    assert!(runtime.enabled);
    assert_eq!(runtime.base_url, "http://127.0.0.1:18825");
    assert_eq!(runtime.schema_version, "v1");
    assert_eq!(runtime.plugin_id, "wendao.memory");
    assert_eq!(runtime.health_route.as_deref(), Some("/healthz"));
    assert_eq!(runtime.scenario_pack.as_deref(), Some("searchinfra"));
    assert_eq!(runtime.timeout_secs, 3);
    assert_eq!(runtime.routes.episodic_recall, "/memory/episodic_recall");
    assert_eq!(runtime.routes.memory_gate_score, "/memory/gate_score");
    assert_eq!(runtime.routes.memory_plan_tuning, "/memory/plan_tuning");
    assert_eq!(runtime.routes.memory_calibration, "/memory/calibration");

    Ok(())
}

#[test]
#[serial]
fn resolve_memory_julia_compute_bindings_materializes_all_profiles()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        r#"[memory.julia_compute]
enabled = true
base_url = "http://127.0.0.1:18825"
schema_version = "v1"
plugin_id = "wendao.memory"
health_route = "/healthz"
timeout_secs = 3

[memory.julia_compute.routes]
episodic_recall = "/memory/episodic_recall"
memory_gate_score = "/memory/gate_score"
memory_plan_tuning = "/memory/plan_tuning"
memory_calibration = "/memory/calibration"
"#,
    )?;
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());

    let bindings = resolve_memory_julia_compute_bindings()?;
    assert_eq!(bindings.len(), 4);
    assert_eq!(bindings[0].selector.provider.0, "wendao.memory");
    assert_eq!(bindings[0].selector.capability_id.0, "episodic_recall");
    assert_eq!(
        bindings[0].endpoint.base_url.as_deref(),
        Some("http://127.0.0.1:18825")
    );
    assert_eq!(
        bindings[0].endpoint.health_route.as_deref(),
        Some("/healthz")
    );
    assert_eq!(
        bindings[1].endpoint.route.as_deref(),
        Some("/memory/gate_score")
    );
    assert_eq!(
        bindings[2].endpoint.route.as_deref(),
        Some("/memory/plan_tuning")
    );
    assert_eq!(
        bindings[3].endpoint.route.as_deref(),
        Some("/memory/calibration")
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn compute_client_fetches_episodic_recall_rows_from_resolved_runtime()
-> Result<(), Box<dyn std::error::Error>> {
    let route = "/memory/episodic_recall";
    let (base_url, server) = spawn_memory_service(episodic_recall_response_batch()).await;
    let temp = write_memory_runtime_override(&base_url, route)?;
    let config_path = temp.path().join("wendao.toml");
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());

    let rows = ComputeClient::configured()?
        .fetch_episodic_recall_score_rows_from_projection(
            &EpisodicRecallQueryInputs {
                query_id: "query-1".to_string(),
                scenario_pack: Some("searchinfra".to_string()),
                query_text: Some("fix memory lane".to_string()),
                query_embedding: vec![0.1, 0.2, 0.3],
                k1: 1.0,
                k2: 0.5,
                lambda: 0.6,
                min_score: 0.1,
            },
            &[MemoryProjectionRow {
                episode_id: "episode-1".to_string().into(),
                scope: "repo".to_string(),
                intent_embedding: vec![0.1, 0.2, 0.3],
                q_value: 0.7,
                success_count: 3,
                failure_count: 1,
                retrieval_count: 4,
                created_at_ms: 100.into(),
                updated_at_ms: 200.into(),
            }],
        )
        .await?;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].candidate_id, "episode-1");
    server.abort();
    Ok(())
}

#[tokio::test]
#[serial]
async fn compute_client_fetches_gate_score_rows_from_resolved_runtime()
-> Result<(), Box<dyn std::error::Error>> {
    let route = "/memory/gate_score";
    let (base_url, server) = spawn_memory_service(gate_score_response_batch()).await;
    let temp = write_memory_runtime_override(&base_url, route)?;
    let config_path = temp.path().join("wendao.toml");
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());

    let rows = ComputeClient::configured()?
        .fetch_gate_score_recommendation_rows_from_evidence(&[MemoryGateScoreEvidenceRow {
            memory_id: "memory-1".to_string(),
            scenario_pack: Some("searchinfra".to_string()),
            ledger: MemoryUtilityLedger {
                react_revalidation_score: 0.9,
                graph_consistency_score: 0.8,
                omega_alignment_score: 0.85,
                ttl_score: 0.7,
                utility_score: 0.78,
                q_value: 0.75,
                usage_count: 4,
                failure_rate: 0.25,
            },
            current_state: MemoryLifecycleState::Active,
        }])
        .await?;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].verdict, "retain");
    server.abort();
    Ok(())
}

#[tokio::test]
#[serial]
async fn compute_client_fetches_plan_tuning_rows_from_resolved_runtime()
-> Result<(), Box<dyn std::error::Error>> {
    let route = "/memory/plan_tuning";
    let (base_url, server) = spawn_memory_service(plan_tuning_response_batch()).await;
    let temp = write_memory_runtime_override(&base_url, route)?;
    let config_path = temp.path().join("wendao.toml");
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());

    let rows = ComputeClient::configured()?
        .fetch_plan_tuning_advice_rows_from_inputs(&[MemoryPlanTuningInputs {
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
        }])
        .await?;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].next_k1, 12);
    server.abort();
    Ok(())
}

#[tokio::test]
#[serial]
async fn compute_client_fetches_calibration_rows_from_resolved_runtime()
-> Result<(), Box<dyn std::error::Error>> {
    let route = "/memory/calibration";
    let (base_url, server) = spawn_memory_service(calibration_response_batch()).await;
    let temp = write_memory_runtime_override(&base_url, route)?;
    let config_path = temp.path().join("wendao.toml");
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());

    let rows = ComputeClient::configured()?
        .fetch_calibration_artifact_rows_from_inputs(&[MemoryCalibrationInputs {
            calibration_job_id: "calibration-1".to_string(),
            scenario_pack: Some("searchinfra".to_string()),
            dataset_ref: "dataset://memory/searchinfra/latest".to_string(),
            objective: "maximize_precision".to_string(),
            hyperparam_config: "{\"max_iter\":32}".to_string(),
        }])
        .await?;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].artifact_ref, "artifact://memory/calibration-1");
    server.abort();
    Ok(())
}

#[tokio::test]
#[serial]
async fn configured_compute_client_errors_when_runtime_is_disabled()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        r"[memory.julia_compute]
enabled = false
",
    )?;
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());

    let error = ComputeClient::configured().err().ok_or_else(|| {
        std::io::Error::other(
            "disabled runtime should return a config-load error instead of succeeding",
        )
    })?;
    assert!(matches!(
        error,
        RepoIntelligenceError::ConfigLoad { ref message }
            if message.contains("memory Julia compute runtime is disabled")
    ));
    Ok(())
}
