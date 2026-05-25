use xiuxian_julia_runtime::wendao::link_graph::{
    DEFAULT_JULIA_SEARCH_EXAMPLE_CONFIG_PATH, DEFAULT_JULIA_SEARCH_LAUNCHER_PATH,
    build_rerank_provider_binding, julia_deployment_artifact_selector,
    julia_rerank_provider_selector,
};

use crate::test_support::{
    LinkedBuiltinJuliaRerankEndpoint, linked_builtin_julia_deployment_artifact_selector,
    linked_builtin_julia_rerank_provider_binding_with_endpoint,
    linked_builtin_julia_rerank_provider_selector, linked_builtin_julia_search_example_config_path,
    linked_builtin_julia_search_launcher_path,
};

#[test]
fn linked_builtin_host_test_helpers_match_julia_compatibility_helpers() {
    assert_eq!(
        linked_builtin_julia_search_example_config_path(),
        DEFAULT_JULIA_SEARCH_EXAMPLE_CONFIG_PATH
    );
    assert_eq!(
        linked_builtin_julia_search_launcher_path(),
        DEFAULT_JULIA_SEARCH_LAUNCHER_PATH
    );
    assert_eq!(
        linked_builtin_julia_rerank_provider_selector(),
        julia_rerank_provider_selector()
    );
    assert_eq!(
        linked_builtin_julia_deployment_artifact_selector(),
        julia_deployment_artifact_selector()
    );
    assert_eq!(
        linked_builtin_julia_rerank_provider_binding_with_endpoint(
            &LinkedBuiltinJuliaRerankEndpoint {
                base_url: "http://127.0.0.1:8090".to_string(),
                route: "/custom-rerank".to_string(),
                health_route: "/healthz".to_string(),
                schema_version: "v1".to_string(),
                timeout_secs: 15,
            },
        ),
        build_rerank_provider_binding(
            &xiuxian_julia_runtime::wendao::link_graph::LinkGraphJuliaRerankRuntimeConfig {
                base_url: Some("http://127.0.0.1:8090".to_string().into()),
                route: Some("/custom-rerank".to_string().into()),
                health_route: Some("/healthz".to_string().into()),
                schema_version: Some("v1".to_string().into()),
                timeout_secs: Some(15_u64.into()),
                service_mode: None,
                search_config_path: None,
                vector_weight: None,
                similarity_weight: None,
            }
        )
    );
}
