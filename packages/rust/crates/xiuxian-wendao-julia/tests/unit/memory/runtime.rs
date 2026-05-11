use super::{build_memory_julia_compute_binding, build_memory_julia_compute_bindings};
use crate::memory::MemoryJuliaComputeProfile;
use xiuxian_wendao_runtime::config::MemoryJuliaComputeRuntimeConfig;

#[test]
fn build_memory_julia_compute_bindings_skips_disabled_runtime() {
    let runtime = MemoryJuliaComputeRuntimeConfig::default();
    let bindings = build_memory_julia_compute_bindings(&runtime)
        .unwrap_or_else(|error| panic!("bindings should resolve: {error}"));
    assert!(bindings.is_empty());
}

#[test]
fn build_memory_julia_compute_bindings_materialize_all_profiles() {
    let mut runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };
    runtime.plugin_id = "wendao.memory".into();
    runtime.base_url = "http://127.0.0.1:18825".to_string();
    runtime.health_route = Some("/healthz".to_string());
    runtime.max_in_flight_requests = 9;
    runtime.routes.episodic_recall = "/memory/episodic_recall".to_string();
    runtime.routes.memory_gate_score = "/memory/gate_score".to_string();
    runtime.routes.memory_plan_tuning = "/memory/plan_tuning".to_string();
    runtime.routes.memory_calibration = "/memory/calibration".to_string();

    let bindings = build_memory_julia_compute_bindings(&runtime)
        .unwrap_or_else(|error| panic!("bindings should resolve: {error}"));
    assert_eq!(bindings.len(), 4);
    assert_eq!(bindings[0].selector.capability_id.0, "episodic_recall");
    assert_eq!(bindings[1].selector.capability_id.0, "memory_gate_score");
    assert_eq!(bindings[2].selector.capability_id.0, "memory_plan_tuning");
    assert_eq!(bindings[3].selector.capability_id.0, "memory_calibration");
    assert_eq!(bindings[0].selector.provider.0, "wendao.memory");
    assert_eq!(
        bindings[0].endpoint.route.as_deref(),
        Some("/memory/episodic_recall")
    );
    assert_eq!(
        bindings[0].endpoint.health_route.as_deref(),
        Some("/healthz")
    );
    assert_eq!(bindings[0].endpoint.max_in_flight_requests, Some(9));
    assert_eq!(
        bindings[3].endpoint.route.as_deref(),
        Some("/memory/calibration")
    );
}

#[test]
fn build_memory_julia_compute_binding_rejects_invalid_runtime_values() {
    let mut runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };
    runtime.plugin_id = "  ".into();
    let Err(error) =
        build_memory_julia_compute_binding(&runtime, MemoryJuliaComputeProfile::EpisodicRecall)
    else {
        panic!("blank provider id should fail");
    };
    let message = error.to_string();
    assert!(message.contains("plugin_id"));

    runtime.plugin_id = "wendao.memory".into();
    runtime.health_route = Some("/".to_string());
    let Err(error) =
        build_memory_julia_compute_binding(&runtime, MemoryJuliaComputeProfile::EpisodicRecall)
    else {
        panic!("invalid health_route should fail");
    };
    assert!(error.to_string().contains("health_route"));

    runtime.health_route = None;
    runtime.routes.episodic_recall = "/".to_string();
    let Err(error) =
        build_memory_julia_compute_binding(&runtime, MemoryJuliaComputeProfile::EpisodicRecall)
    else {
        panic!("invalid route should fail");
    };
    assert!(error.to_string().contains("invalid route"));
}
