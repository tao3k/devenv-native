use super::{
    DEFAULT_MEMORY_JULIA_COMPUTE_BASE_URL, DEFAULT_MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_ROUTE,
    DEFAULT_MEMORY_JULIA_COMPUTE_MAX_IN_FLIGHT_REQUESTS, DEFAULT_MEMORY_JULIA_COMPUTE_PLUGIN_ID,
    DEFAULT_MEMORY_JULIA_COMPUTE_SCHEMA_VERSION, DEFAULT_MEMORY_JULIA_COMPUTE_TIMEOUT_SECS,
    MemoryJuliaComputeFallbackMode, MemoryJuliaComputeServiceMode,
    resolve_memory_julia_compute_runtime_with_settings,
};
use crate::config::test_support;
use std::fs;

#[test]
fn memory_julia_compute_runtime_reads_override_values() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        r#"[memory.julia_compute]
enabled = true
base_url = "grpc://127.0.0.1:18825"
schema_version = "v2"
plugin_id = "wendao.memory.shadow"
health_route = "/healthz"
service_mode = "table"
scenario_pack = "searchinfra"
timeout_secs = 3
max_in_flight_requests = 7
fallback_mode = "rust"
shadow_compare = false

[memory.julia_compute.routes]
episodic_recall = "/memory/custom_recall"
memory_gate_score = "/memory/custom_gate_score"
memory_plan_tuning = "/memory/custom_plan_tuning"
memory_calibration = "/memory/custom_calibration"
"#,
    )?;

    let settings = test_support::load_test_settings_from_path(&config_path)?;
    let runtime = resolve_memory_julia_compute_runtime_with_settings(&settings);

    assert!(runtime.enabled);
    assert_eq!(runtime.base_url, "grpc://127.0.0.1:18825");
    assert_eq!(runtime.schema_version, "v2");
    assert_eq!(runtime.plugin_id, "wendao.memory.shadow");
    assert_eq!(runtime.health_route.as_deref(), Some("/healthz"));
    assert_eq!(runtime.service_mode, MemoryJuliaComputeServiceMode::Table);
    assert_eq!(runtime.scenario_pack.as_deref(), Some("searchinfra"));
    assert_eq!(runtime.timeout_secs, 3);
    assert_eq!(runtime.max_in_flight_requests, 7);
    assert_eq!(runtime.fallback_mode, MemoryJuliaComputeFallbackMode::Rust);
    assert!(!runtime.shadow_compare);
    assert_eq!(runtime.routes.episodic_recall, "/memory/custom_recall");
    assert_eq!(
        runtime.routes.memory_gate_score,
        "/memory/custom_gate_score"
    );
    assert_eq!(
        runtime.routes.memory_plan_tuning,
        "/memory/custom_plan_tuning"
    );
    assert_eq!(
        runtime.routes.memory_calibration,
        "/memory/custom_calibration"
    );

    Ok(())
}

#[test]
fn memory_julia_compute_runtime_falls_back_on_blank_or_invalid_values()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        r#"[memory.julia_compute]
base_url = "   "
schema_version = "   "
plugin_id = ""
health_route = " "
service_mode = "invalid"
timeout_secs = 0
max_in_flight_requests = 0
fallback_mode = "invalid"
shadow_compare = true

[memory.julia_compute.routes]
episodic_recall = "   "
"#,
    )?;

    let settings = test_support::load_test_settings_from_path(&config_path)?;
    let runtime = resolve_memory_julia_compute_runtime_with_settings(&settings);

    assert!(!runtime.enabled);
    assert_eq!(runtime.base_url, DEFAULT_MEMORY_JULIA_COMPUTE_BASE_URL);
    assert_eq!(
        runtime.schema_version,
        DEFAULT_MEMORY_JULIA_COMPUTE_SCHEMA_VERSION
    );
    assert_eq!(runtime.plugin_id, DEFAULT_MEMORY_JULIA_COMPUTE_PLUGIN_ID);
    assert_eq!(runtime.health_route, None);
    assert_eq!(runtime.service_mode, MemoryJuliaComputeServiceMode::Stream);
    assert_eq!(
        runtime.timeout_secs,
        DEFAULT_MEMORY_JULIA_COMPUTE_TIMEOUT_SECS
    );
    assert_eq!(
        runtime.max_in_flight_requests,
        DEFAULT_MEMORY_JULIA_COMPUTE_MAX_IN_FLIGHT_REQUESTS
    );
    assert_eq!(runtime.fallback_mode, MemoryJuliaComputeFallbackMode::Rust);
    assert!(runtime.shadow_compare);
    assert_eq!(
        runtime.routes.episodic_recall,
        DEFAULT_MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_ROUTE
    );

    Ok(())
}
