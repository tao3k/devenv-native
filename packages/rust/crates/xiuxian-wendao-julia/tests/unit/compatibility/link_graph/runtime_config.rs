#[test]
fn runtime_config_builds_provider_binding_and_artifact_payload() {
    let runtime = LinkGraphJuliaRerankRuntimeConfig {
        base_url: Some("http://127.0.0.1:8088".into()),
        route: Some(DEFAULT_JULIA_RERANK_FLIGHT_ROUTE.into()),
        health_route: Some("/healthz".into()),
        schema_version: Some("v1".into()),
        timeout_secs: Some(15_u64.into()),
        service_mode: Some("stream".into()),
        search_config_path: Some(DEFAULT_JULIA_SEARCH_EXAMPLE_CONFIG_PATH.into()),
        vector_weight: Some(0.2),
        similarity_weight: Some(0.8),
    };

    let descriptor = runtime.search_service_descriptor();
    let provider_descriptor = runtime.provider_launch_descriptor();
    assert_eq!(provider_descriptor, descriptor);
    assert_eq!(descriptor.service_mode.as_deref(), Some("stream"));
    assert_eq!(
        descriptor.search_config_path.as_deref(),
        Some(DEFAULT_JULIA_SEARCH_EXAMPLE_CONFIG_PATH)
    );
    assert_eq!(runtime.vector_weight, Some(0.2));
    assert_eq!(runtime.similarity_weight, Some(0.8));

    let manifest = runtime.search_launch_manifest();
    let launch_spec = runtime.plugin_launch_spec();
    assert_eq!(manifest.launcher_path, launch_spec.launcher_path);
    assert_eq!(manifest.args, launch_spec.args);
    assert_eq!(manifest.launcher_path, DEFAULT_JULIA_SEARCH_LAUNCHER_PATH);
    assert_eq!(
        manifest.args,
        vec![
            "--mode",
            "stream",
            "--config",
            DEFAULT_JULIA_SEARCH_EXAMPLE_CONFIG_PATH,
        ]
    );

    let binding = build_rerank_provider_binding(&runtime);
    let Some(direct_binding) = runtime.rerank_provider_binding() else {
        panic!("direct binding");
    };
    let Some(binding_launch) = binding.launch.clone() else {
        panic!("launch");
    };
    assert_eq!(direct_binding, binding);
    assert_eq!(binding.selector, julia_rerank_provider_selector());
    assert_eq!(
        binding.endpoint.base_url.as_deref(),
        Some("http://127.0.0.1:8088")
    );
    assert_eq!(
        binding.transport,
        xiuxian_wendao_core::transport::PluginTransportKind::ArrowFlight
    );
    assert_eq!(
        binding_launch.launcher_path,
        DEFAULT_JULIA_SEARCH_LAUNCHER_PATH
    );

    let artifact = runtime.deployment_artifact();
    let artifact_payload = runtime.plugin_artifact_payload();
    let artifact_selector = julia_deployment_artifact_selector();
    assert_eq!(
        artifact.artifact_schema_version,
        DEFAULT_JULIA_DEPLOYMENT_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(artifact_payload.plugin_id, artifact_selector.plugin_id);
    assert_eq!(artifact_payload.artifact_id, artifact_selector.artifact_id);
    assert_eq!(artifact.base_url.as_deref(), Some("http://127.0.0.1:8088"));
    assert_eq!(
        artifact.route.as_deref(),
        Some(DEFAULT_JULIA_RERANK_FLIGHT_ROUTE)
    );
    assert_eq!(artifact.health_route.as_deref(), Some("/healthz"));
    assert_eq!(artifact.schema_version.as_deref(), Some("v1"));
    assert_eq!(artifact.timeout_secs, Some(15));
    assert_eq!(artifact.launch, manifest);
}

#[test]
fn runtime_config_resolves_from_settings_and_env_lookup() -> Result<(), Box<dyn std::error::Error>>
{
    let settings: Value = serde_yaml::from_str(&format!(
        r#"
link_graph:
  retrieval:
    julia_rerank:
      base_url: "http://127.0.0.1:8088"
      route: " /rerank "
      schema_version: "v1"
      timeout_secs: 15
      service_mode: "stream"
      search_config_path: "{DEFAULT_JULIA_SEARCH_EXAMPLE_CONFIG_PATH}"
      vector_weight: 0.2
"#
    ))?;

    let runtime = LinkGraphJuliaRerankRuntimeConfig::resolve_with_env_lookup(&settings, |name| {
        if name == LINK_GRAPH_JULIA_RERANK_HEALTH_ROUTE_ENV {
            Some("/healthz".to_string())
        } else if name == LINK_GRAPH_JULIA_RERANK_SIMILARITY_WEIGHT_ENV {
            Some("0.8".to_string())
        } else {
            None
        }
    });

    assert_eq!(runtime.base_url.as_deref(), Some("http://127.0.0.1:8088"));
    assert_eq!(runtime.route.as_deref(), Some("/rerank"));
    assert_eq!(runtime.health_route.as_deref(), Some("/healthz"));
    assert_eq!(runtime.schema_version.as_deref(), Some("v1"));
    assert_eq!(runtime.timeout_secs.map(|seconds| seconds.value()), Some(15));
    assert_eq!(runtime.service_mode.as_deref(), Some("stream"));
    assert_eq!(
        runtime.search_config_path.as_deref(),
        Some(DEFAULT_JULIA_SEARCH_EXAMPLE_CONFIG_PATH)
    );
    assert_eq!(runtime.vector_weight, Some(0.2));
    assert_eq!(runtime.similarity_weight, Some(0.8));

    Ok(())
}

#[test]
fn runtime_config_resolution_prefers_settings_over_env_lookup()
-> Result<(), Box<dyn std::error::Error>> {
    let settings: Value = serde_yaml::from_str(
        r#"
link_graph:
  retrieval:
    julia_rerank:
      base_url: "http://127.0.0.1:8088"
      route: "/rerank"
      health_route: "/healthz"
      schema_version: "v1"
      timeout_secs: 15
      service_mode: "stream"
      search_config_path: "config/search.toml"
      vector_weight: 0.2
      similarity_weight: 0.8
"#,
    )?;

    let runtime = LinkGraphJuliaRerankRuntimeConfig::resolve_with_env_lookup(&settings, |name| {
        let value = if name == LINK_GRAPH_JULIA_RERANK_BASE_URL_ENV {
            "http://127.0.0.1:9999"
        } else if name == LINK_GRAPH_JULIA_RERANK_ROUTE_ENV {
            "/env-rerank"
        } else if name == LINK_GRAPH_JULIA_RERANK_HEALTH_ROUTE_ENV {
            "/env-health"
        } else if name == LINK_GRAPH_JULIA_RERANK_SCHEMA_VERSION_ENV {
            "v2"
        } else if name == LINK_GRAPH_JULIA_RERANK_TIMEOUT_SECS_ENV {
            "77"
        } else if name == LINK_GRAPH_JULIA_RERANK_SERVICE_MODE_ENV {
            "batch"
        } else if name == LINK_GRAPH_JULIA_RERANK_SEARCH_CONFIG_PATH_ENV {
            "config/env.toml"
        } else if name == LINK_GRAPH_JULIA_RERANK_VECTOR_WEIGHT_ENV {
            "0.7"
        } else if name == LINK_GRAPH_JULIA_RERANK_SIMILARITY_WEIGHT_ENV {
            "0.3"
        } else {
            return None;
        };
        Some(value.to_string())
    });

    assert_eq!(runtime.base_url.as_deref(), Some("http://127.0.0.1:8088"));
    assert_eq!(runtime.route.as_deref(), Some("/rerank"));
    assert_eq!(runtime.health_route.as_deref(), Some("/healthz"));
    assert_eq!(runtime.schema_version.as_deref(), Some("v1"));
    assert_eq!(runtime.timeout_secs.map(|seconds| seconds.value()), Some(15));
    assert_eq!(runtime.service_mode.as_deref(), Some("stream"));
    assert_eq!(
        runtime.search_config_path.as_deref(),
        Some("config/search.toml")
    );
    assert_eq!(runtime.vector_weight, Some(0.2));
    assert_eq!(runtime.similarity_weight, Some(0.8));

    Ok(())
}

#[test]
fn rerank_runtime_converts_into_generic_binding() {
    let binding = build_rerank_provider_binding(&LinkGraphJuliaRerankRuntimeConfig {
        base_url: Some("http://127.0.0.1:8088".into()),
        route: Some("/rerank".into()),
        health_route: Some("/healthz".into()),
        schema_version: Some("v2".into()),
        timeout_secs: Some(15_u64.into()),
        service_mode: Some("stream".into()),
        search_config_path: Some("config/search.toml".into()),
        vector_weight: Some(0.7),
        similarity_weight: Some(0.3),
    });
    let selector = julia_rerank_provider_selector();

    assert_eq!(binding.selector, selector);
    assert_eq!(
        binding.transport,
        xiuxian_wendao_core::transport::PluginTransportKind::ArrowFlight
    );
    assert_eq!(binding.contract_version.0, "v2");
    assert_eq!(
        binding.endpoint.base_url.as_deref(),
        Some("http://127.0.0.1:8088")
    );
    assert_eq!(binding.endpoint.route.as_deref(), Some("/rerank"));
    assert_eq!(binding.endpoint.health_route.as_deref(), Some("/healthz"));
    assert_eq!(binding.endpoint.timeout_secs, Some(15));
    let Some(launch) = binding.launch else {
        panic!("launch");
    };
    assert!(launch.args.iter().any(|value| value == "--mode"));
    assert!(launch.args.iter().any(|value| value == "--config"));
}
