#[cfg(feature = "julia")]
use crate::link_graph::runtime_config::resolve_link_graph_retrieval_policy_runtime;
#[cfg(feature = "julia")]
use crate::link_graph::runtime_config::{
    resolve_link_graph_rerank_binding, resolve_link_graph_rerank_flight_runtime_settings,
    resolve_link_graph_rerank_schema_version, resolve_link_graph_rerank_score_weights,
};
#[cfg(feature = "julia")]
use serial_test::serial;
#[cfg(feature = "julia")]
use xiuxian_wendao_builtin::{
    linked_builtin_julia_rerank_provider_selector, linked_builtin_julia_search_launcher_path,
};
#[cfg(feature = "julia")]
use xiuxian_wendao_runtime::transport::RerankScoreWeights;

#[cfg(feature = "julia")]
use super::support::configure_julia_rerank_runtime_fixture;

#[cfg(feature = "julia")]
#[test]
#[serial]
fn test_retrieval_runtime_resolves_julia_rerank_config() -> Result<(), Box<dyn std::error::Error>> {
    let _temp = configure_julia_rerank_runtime_fixture()?;

    let runtime = resolve_link_graph_retrieval_policy_runtime();
    let Some(binding) = runtime.rerank_binding() else {
        panic!("generic rerank binding");
    };

    assert_eq!(
        binding.selector,
        linked_builtin_julia_rerank_provider_selector()
    );
    assert_eq!(
        binding.endpoint.base_url.as_deref(),
        Some("http://127.0.0.1:8088")
    );
    assert_eq!(binding.endpoint.route.as_deref(), Some("/rerank"));
    assert_eq!(binding.endpoint.health_route.as_deref(), Some("/healthz"));
    assert_eq!(binding.endpoint.timeout_secs, Some(15));
    assert_eq!(
        binding
            .launch
            .as_ref()
            .map(|launch| launch.launcher_path.as_str()),
        Some(linked_builtin_julia_search_launcher_path())
    );
    assert_eq!(runtime.rerank_schema_version().as_deref(), Some("v1"));
    let score_weights = match RerankScoreWeights::new(0.2, 0.8) {
        Ok(weights) => weights,
        Err(error) => panic!("valid weight fixture should construct: {error}"),
    };
    assert_eq!(runtime.rerank_score_weights(), Some(score_weights));

    Ok(())
}

#[cfg(feature = "julia")]
#[test]
#[serial]
fn test_retrieval_runtime_projects_julia_rerank_host_helpers()
-> Result<(), Box<dyn std::error::Error>> {
    let _temp = configure_julia_rerank_runtime_fixture()?;

    let runtime = resolve_link_graph_retrieval_policy_runtime();
    let Some(score_weights) = resolve_link_graph_rerank_score_weights() else {
        panic!("score weights should resolve");
    };
    assert!((score_weights.vector_weight - 0.2).abs() < f64::EPSILON);
    assert!((score_weights.semantic_weight - 0.8).abs() < f64::EPSILON);
    assert_eq!(
        resolve_link_graph_rerank_schema_version().as_deref(),
        Some("v1")
    );
    let flight_settings = resolve_link_graph_rerank_flight_runtime_settings();
    assert_eq!(flight_settings.schema_version.as_deref(), Some("v1"));
    let Some(flight_weights) = flight_settings.score_weights else {
        panic!("flight score weights should resolve");
    };
    assert!((flight_weights.vector_weight - 0.2).abs() < f64::EPSILON);
    assert!((flight_weights.semantic_weight - 0.8).abs() < f64::EPSILON);
    let Some(binding) = runtime.rerank_binding() else {
        panic!("generic rerank binding");
    };
    assert_eq!(
        binding.selector,
        linked_builtin_julia_rerank_provider_selector()
    );
    assert_eq!(
        binding.endpoint.base_url.as_deref(),
        Some("http://127.0.0.1:8088")
    );
    assert_eq!(binding.endpoint.route.as_deref(), Some("/rerank"));
    assert_eq!(
        binding.transport,
        xiuxian_wendao_core::transport::PluginTransportKind::ArrowFlight
    );
    assert_eq!(binding.endpoint.health_route.as_deref(), Some("/healthz"));
    assert_eq!(binding.endpoint.timeout_secs, Some(15));
    assert_eq!(binding.contract_version.0, "v1");
    assert_eq!(
        binding
            .launch
            .as_ref()
            .map(|launch| launch.launcher_path.as_str()),
        Some(linked_builtin_julia_search_launcher_path())
    );

    let Some(resolved_binding) = resolve_link_graph_rerank_binding() else {
        panic!("resolved generic rerank binding");
    };
    assert_eq!(
        resolved_binding.selector,
        linked_builtin_julia_rerank_provider_selector()
    );
    assert_eq!(
        resolved_binding.endpoint.base_url.as_deref(),
        Some("http://127.0.0.1:8088")
    );

    Ok(())
}
