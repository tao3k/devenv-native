mod julia {
    use xiuxian_julia_core::integration_support::{
        julia_gateway_artifact_base_url, julia_gateway_artifact_expected_json_fragments,
        julia_gateway_artifact_expected_toml_fragments, julia_gateway_artifact_path,
        julia_gateway_artifact_rpc_params_fixture, julia_gateway_artifact_runtime_config_toml,
        julia_gateway_artifact_schema_version, julia_gateway_artifact_selected_transport,
        julia_ui_artifact_payload_fixture,
    };
    use xiuxian_julia_runtime::wendao::link_graph::{
        DEFAULT_JULIA_RERANK_FLIGHT_ROUTE, DEFAULT_JULIA_SEARCH_LAUNCHER_PATH,
        LinkGraphJuliaRerankRuntimeConfig, julia_deployment_artifact_openapi_json_example,
        julia_deployment_artifact_openapi_toml_example, julia_deployment_artifact_selector,
        julia_plugin_artifact_openapi_json_example, julia_plugin_artifact_openapi_toml_example,
    };
    use xiuxian_wendao_core::{
        artifacts::PluginArtifactSelector,
        ids::{ArtifactId, PluginId},
    };

    use crate::{
        linked_builtin_julia_deployment_artifact_openapi_json_example,
        linked_builtin_julia_deployment_artifact_openapi_toml_example,
        linked_builtin_julia_gateway_artifact_base_url,
        linked_builtin_julia_gateway_artifact_expected_json_fragments,
        linked_builtin_julia_gateway_artifact_expected_toml_fragments,
        linked_builtin_julia_gateway_artifact_path, linked_builtin_julia_gateway_artifact_route,
        linked_builtin_julia_gateway_artifact_rpc_params_fixture,
        linked_builtin_julia_gateway_artifact_runtime_config_toml,
        linked_builtin_julia_gateway_artifact_schema_version,
        linked_builtin_julia_gateway_artifact_selected_transport,
        linked_builtin_julia_gateway_artifact_ui_payload_fixture,
        linked_builtin_julia_gateway_launcher_path,
        linked_builtin_plugin_artifact_openapi_json_example,
        linked_builtin_plugin_artifact_openapi_toml_example,
        render_builtin_plugin_artifact_toml_for_selector,
        resolve_builtin_plugin_artifact_for_selector,
    };

    fn julia_runtime_fixture() -> LinkGraphJuliaRerankRuntimeConfig {
        LinkGraphJuliaRerankRuntimeConfig {
            base_url: Some("http://127.0.0.1:8815".to_string().into()),
            route: Some("/rerank".to_string().into()),
            health_route: Some("/healthz".to_string().into()),
            schema_version: Some("v1".to_string().into()),
            timeout_secs: Some(15_u64.into()),
            service_mode: Some("stream".to_string().into()),
            search_config_path: Some("examples/julia.toml".to_string().into()),
            vector_weight: Some(0.2),
            similarity_weight: Some(0.8),
        }
    }

    #[test]
    fn resolve_builtin_plugin_artifact_for_selector_resolves_julia_artifact() {
        let runtime = julia_runtime_fixture();
        let selector = julia_deployment_artifact_selector();

        let artifact = resolve_builtin_plugin_artifact_for_selector(&selector, &runtime)
            .unwrap_or_else(|| panic!("builtin selector should resolve the Julia artifact"));

        assert_eq!(artifact.plugin_id, selector.plugin_id);
        assert_eq!(artifact.artifact_id, selector.artifact_id);
        assert_eq!(
            artifact.endpoint.and_then(|endpoint| endpoint.route),
            Some("/rerank".into())
        );
    }

    #[test]
    fn resolve_builtin_plugin_artifact_for_selector_rejects_unknown_selector() {
        let runtime = julia_runtime_fixture();
        let selector = PluginArtifactSelector {
            plugin_id: PluginId("other".to_string()),
            artifact_id: ArtifactId("deployment".to_string()),
        };

        assert!(
            resolve_builtin_plugin_artifact_for_selector(&selector, &runtime).is_none(),
            "non-builtin selector should be ignored"
        );
    }

    #[test]
    fn render_builtin_plugin_artifact_toml_for_selector_serializes_julia_artifact() {
        let runtime = julia_runtime_fixture();
        let selector = julia_deployment_artifact_selector();

        let rendered = render_builtin_plugin_artifact_toml_for_selector(&selector, &runtime)
            .unwrap_or_else(|error| panic!("builtin artifact render should succeed: {error}"))
            .unwrap_or_else(|| panic!("builtin selector should render the Julia artifact"));

        assert!(
            rendered.contains("artifact_schema_version = \"v1\""),
            "unexpected rendered artifact: {rendered}"
        );
        assert!(
            rendered.contains("route = \"/rerank\""),
            "unexpected rendered artifact: {rendered}"
        );
    }

    #[test]
    fn linked_builtin_openapi_helpers_match_julia_plugin_examples() {
        assert_eq!(
            linked_builtin_plugin_artifact_openapi_json_example(),
            julia_plugin_artifact_openapi_json_example()
        );
        assert_eq!(
            linked_builtin_plugin_artifact_openapi_toml_example(),
            julia_plugin_artifact_openapi_toml_example()
        );
        assert_eq!(
            linked_builtin_julia_deployment_artifact_openapi_json_example(),
            julia_deployment_artifact_openapi_json_example()
        );
        assert_eq!(
            linked_builtin_julia_deployment_artifact_openapi_toml_example()
                .unwrap_or_else(|error| panic!("builtin legacy example should render: {error}")),
            julia_deployment_artifact_openapi_toml_example()
                .unwrap_or_else(|error| panic!("plugin legacy example should render: {error}"))
        );
    }

    #[test]
    fn linked_builtin_gateway_helpers_match_julia_plugin_fixtures() {
        assert_eq!(
            linked_builtin_julia_gateway_artifact_base_url(),
            julia_gateway_artifact_base_url()
        );
        assert_eq!(
            linked_builtin_julia_gateway_artifact_schema_version(),
            julia_gateway_artifact_schema_version()
        );
        assert_eq!(
            linked_builtin_julia_gateway_artifact_selected_transport(),
            julia_gateway_artifact_selected_transport()
        );
        let linked_artifact_path = linked_builtin_julia_gateway_artifact_path();
        let julia_artifact_path = julia_gateway_artifact_path();
        assert_eq!(
            linked_artifact_path.plugin_id,
            julia_artifact_path.plugin_id
        );
        assert_eq!(
            linked_artifact_path.artifact_id,
            julia_artifact_path.artifact_id
        );
        assert_eq!(
            linked_builtin_julia_gateway_artifact_route(),
            DEFAULT_JULIA_RERANK_FLIGHT_ROUTE
        );
        assert_eq!(
            linked_builtin_julia_gateway_launcher_path(),
            DEFAULT_JULIA_SEARCH_LAUNCHER_PATH
        );
        assert_eq!(
            linked_builtin_julia_gateway_artifact_runtime_config_toml(),
            julia_gateway_artifact_runtime_config_toml()
        );
        assert_eq!(
            linked_builtin_julia_gateway_artifact_expected_toml_fragments(),
            julia_gateway_artifact_expected_toml_fragments()
        );
        assert_eq!(
            linked_builtin_julia_gateway_artifact_expected_json_fragments(),
            julia_gateway_artifact_expected_json_fragments()
        );
        assert_eq!(
            linked_builtin_julia_gateway_artifact_rpc_params_fixture(
                Some("json"),
                Some(".run/julia/plugin-artifact.json")
            ),
            julia_gateway_artifact_rpc_params_fixture(
                Some("json"),
                Some(".run/julia/plugin-artifact.json")
            )
        );
        assert_eq!(
            linked_builtin_julia_gateway_artifact_ui_payload_fixture(),
            julia_ui_artifact_payload_fixture()
        );
    }
}
