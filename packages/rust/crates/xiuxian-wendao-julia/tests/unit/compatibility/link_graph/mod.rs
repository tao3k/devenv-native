use serde_yaml::Value;
use std::fs;
use xiuxian_wendao_core::{
    artifacts::{PluginArtifactPayload, PluginLaunchSpec},
    transport::PluginTransportEndpoint,
};

use super::{
    DEFAULT_JULIA_ANALYZER_EXAMPLE_CONFIG_PATH, DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH,
    DEFAULT_JULIA_DEPLOYMENT_ARTIFACT_SCHEMA_VERSION, DEFAULT_JULIA_RERANK_FLIGHT_ROUTE,
    JULIA_DEPLOYMENT_ARTIFACT_ID, JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID, JULIA_PLUGIN_ID,
    JULIA_RERANK_CAPABILITY_ID, LINK_GRAPH_JULIA_RERANK_ANALYZER_CONFIG_PATH_ENV,
    LINK_GRAPH_JULIA_RERANK_ANALYZER_STRATEGY_ENV, LINK_GRAPH_JULIA_RERANK_BASE_URL_ENV,
    LINK_GRAPH_JULIA_RERANK_HEALTH_ROUTE_ENV, LINK_GRAPH_JULIA_RERANK_ROUTE_ENV,
    LINK_GRAPH_JULIA_RERANK_SCHEMA_VERSION_ENV, LINK_GRAPH_JULIA_RERANK_SERVICE_MODE_ENV,
    LINK_GRAPH_JULIA_RERANK_SIMILARITY_WEIGHT_ENV, LINK_GRAPH_JULIA_RERANK_TIMEOUT_SECS_ENV,
    LINK_GRAPH_JULIA_RERANK_VECTOR_WEIGHT_ENV, LinkGraphJuliaAnalyzerLaunchManifest,
    LinkGraphJuliaAnalyzerServiceDescriptor, LinkGraphJuliaDeploymentArtifact,
    LinkGraphJuliaRerankRuntimeConfig, build_rerank_provider_binding,
    julia_deployment_artifact_openapi_example, julia_deployment_artifact_openapi_json_example,
    julia_deployment_artifact_openapi_toml_example, julia_deployment_artifact_selector,
    julia_graph_structural_provider_selector, julia_plugin_artifact_openapi_json_example,
    julia_plugin_artifact_openapi_toml_example, julia_rerank_provider_selector,
    render_julia_plugin_artifact_toml_for_selector,
    resolve_julia_plugin_artifact_payload_for_selector,
};

#[test]
fn selectors_keep_stable_julia_ids() {
    let provider = julia_rerank_provider_selector();
    let graph_structural = julia_graph_structural_provider_selector();
    let artifact = julia_deployment_artifact_selector();

    assert_eq!(provider.provider.0, JULIA_PLUGIN_ID);
    assert_eq!(provider.capability_id.0, JULIA_RERANK_CAPABILITY_ID);
    assert_eq!(graph_structural.provider.0, JULIA_PLUGIN_ID);
    assert_eq!(
        graph_structural.capability_id.0,
        JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID
    );
    assert_eq!(artifact.plugin_id.0, JULIA_PLUGIN_ID);
    assert_eq!(artifact.artifact_id.0, JULIA_DEPLOYMENT_ARTIFACT_ID);
}

#[test]
fn launch_manifest_round_trips_plugin_launch_spec() {
    let launch = LinkGraphJuliaAnalyzerLaunchManifest {
        launcher_path: DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH.to_string(),
        args: vec!["--service-mode".to_string(), "stream".to_string()],
    };

    let spec: PluginLaunchSpec = launch.clone().into();
    let roundtrip = LinkGraphJuliaAnalyzerLaunchManifest::from(spec);

    assert_eq!(roundtrip, launch);
}

#[test]
fn service_descriptor_builds_plugin_launch_spec() {
    let descriptor = LinkGraphJuliaAnalyzerServiceDescriptor {
        service_mode: Some("stream".to_string()),
        analyzer_config_path: Some(DEFAULT_JULIA_ANALYZER_EXAMPLE_CONFIG_PATH.to_string()),
        analyzer_strategy: Some("similarity_only".to_string()),
        vector_weight: Some(0.2),
        similarity_weight: Some(0.8),
    };

    let spec = descriptor.plugin_launch_spec(DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH);

    assert_eq!(spec.launcher_path, DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH);
    assert_eq!(
        spec.args,
        vec![
            "--service-mode",
            "stream",
            "--analyzer-config",
            DEFAULT_JULIA_ANALYZER_EXAMPLE_CONFIG_PATH,
            "--analyzer-strategy",
            "similarity_only",
            "--vector-weight",
            "0.2",
            "--similarity-weight",
            "0.8",
        ]
    );
}

#[test]
fn deployment_artifact_round_trips_plugin_artifact_payload() {
    let artifact = LinkGraphJuliaDeploymentArtifact {
        artifact_schema_version: "v1".to_string(),
        generated_at: "2026-03-28T12:34:56Z".to_string(),
        base_url: Some("http://127.0.0.1:8080".to_string()),
        route: Some(DEFAULT_JULIA_RERANK_FLIGHT_ROUTE.to_string()),
        health_route: Some("/health".to_string()),
        schema_version: Some("v1".to_string()),
        timeout_secs: Some(15),
        launch: LinkGraphJuliaAnalyzerLaunchManifest {
            launcher_path: DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH.to_string(),
            args: vec!["--service-mode".to_string(), "stream".to_string()],
        },
    };

    let payload: PluginArtifactPayload = artifact.clone().into();
    let Some(payload_launch) = payload.launch.clone() else {
        panic!("payload launch should exist");
    };

    assert_eq!(payload.plugin_id.0, JULIA_PLUGIN_ID);
    assert_eq!(payload.artifact_id.0, JULIA_DEPLOYMENT_ARTIFACT_ID);
    assert_eq!(payload_launch.launcher_path, artifact.launch.launcher_path);

    let roundtrip = LinkGraphJuliaDeploymentArtifact::from(PluginArtifactPayload {
        plugin_id: payload.plugin_id,
        artifact_id: payload.artifact_id,
        artifact_schema_version: payload.artifact_schema_version,
        generated_at: payload.generated_at,
        endpoint: Some(PluginTransportEndpoint {
            base_url: Some("http://127.0.0.1:8080".to_string()),
            route: Some(DEFAULT_JULIA_RERANK_FLIGHT_ROUTE.to_string()),
            health_route: Some("/health".to_string()),
            timeout_secs: Some(15),
            max_in_flight_requests: None,
        }),
        schema_version: Some("v1".to_string()),
        launch: Some(PluginLaunchSpec {
            launcher_path: DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH.to_string(),
            args: vec!["--service-mode".to_string(), "stream".to_string()],
        }),
        selected_transport: None,
        fallback_from: None,
        fallback_reason: None,
    });

    assert_eq!(roundtrip.artifact_schema_version, "v1");
    assert_eq!(roundtrip.schema_version.as_deref(), Some("v1"));
    assert_eq!(roundtrip.timeout_secs, Some(15));
}

#[test]
fn runtime_config_builds_provider_binding_and_artifact_payload() {
    let runtime = LinkGraphJuliaRerankRuntimeConfig {
        base_url: Some("http://127.0.0.1:8088".to_string()),
        route: Some(DEFAULT_JULIA_RERANK_FLIGHT_ROUTE.to_string()),
        health_route: Some("/healthz".to_string()),
        schema_version: Some("v1".to_string()),
        timeout_secs: Some(15),
        service_mode: Some("stream".to_string()),
        analyzer_config_path: Some(DEFAULT_JULIA_ANALYZER_EXAMPLE_CONFIG_PATH.to_string()),
        analyzer_strategy: Some("similarity_only".to_string()),
        vector_weight: Some(0.2),
        similarity_weight: Some(0.8),
    };

    let descriptor = runtime.analyzer_service_descriptor();
    let provider_descriptor = runtime.provider_launch_descriptor();
    assert_eq!(provider_descriptor, descriptor);
    assert_eq!(descriptor.service_mode.as_deref(), Some("stream"));
    assert_eq!(
        descriptor.analyzer_config_path.as_deref(),
        Some(DEFAULT_JULIA_ANALYZER_EXAMPLE_CONFIG_PATH)
    );
    assert_eq!(
        descriptor.analyzer_strategy.as_deref(),
        Some("similarity_only")
    );
    assert_eq!(descriptor.vector_weight, Some(0.2));
    assert_eq!(descriptor.similarity_weight, Some(0.8));

    let manifest = runtime.analyzer_launch_manifest();
    let launch_spec = runtime.plugin_launch_spec();
    assert_eq!(manifest.launcher_path, launch_spec.launcher_path);
    assert_eq!(manifest.args, launch_spec.args);
    assert_eq!(manifest.launcher_path, DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH);
    assert_eq!(
        manifest.args,
        vec![
            "--service-mode",
            "stream",
            "--analyzer-config",
            DEFAULT_JULIA_ANALYZER_EXAMPLE_CONFIG_PATH,
            "--analyzer-strategy",
            "similarity_only",
            "--vector-weight",
            "0.2",
            "--similarity-weight",
            "0.8",
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
        DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH
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
fn openapi_examples_keep_generic_plugin_artifact_contract() {
    let json = julia_plugin_artifact_openapi_json_example();
    let toml = julia_plugin_artifact_openapi_toml_example();

    assert_eq!(json["pluginId"], JULIA_PLUGIN_ID);
    assert_eq!(json["artifactId"], JULIA_DEPLOYMENT_ARTIFACT_ID);
    assert_eq!(json["schemaVersion"], "v1");
    assert_eq!(json["route"], DEFAULT_JULIA_RERANK_FLIGHT_ROUTE);
    assert!(toml.contains("plugin_id = \"xiuxian-wendao-julia\""));
    assert!(toml.contains("artifact_id = \"deployment\""));
    assert!(toml.contains("route = \"/rerank\""));
}

#[test]
fn openapi_examples_keep_legacy_deployment_artifact_contract() {
    let example = julia_deployment_artifact_openapi_example();
    let json = julia_deployment_artifact_openapi_json_example();
    let toml = julia_deployment_artifact_openapi_toml_example()
        .unwrap_or_else(|error| panic!("render deployment artifact example: {error}"));

    assert_eq!(
        example.artifact_schema_version,
        DEFAULT_JULIA_DEPLOYMENT_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(json["artifactSchemaVersion"], "v1");
    assert_eq!(json["route"], DEFAULT_JULIA_RERANK_FLIGHT_ROUTE);
    assert_eq!(json["healthRoute"], "/healthz");
    assert!(toml.contains("artifact_schema_version = \"v1\""));
    assert!(toml.contains("route = \"/rerank\""));
    assert!(toml.contains("health_route = \"/healthz\""));
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
      analyzer_config_path: "{DEFAULT_JULIA_ANALYZER_EXAMPLE_CONFIG_PATH}"
      analyzer_strategy: "similarity_only"
      vector_weight: 0.2
"#
    ))?;

    let runtime =
        LinkGraphJuliaRerankRuntimeConfig::resolve_with_env_lookup(&settings, |name| match name {
            LINK_GRAPH_JULIA_RERANK_HEALTH_ROUTE_ENV => Some("/healthz".to_string()),
            LINK_GRAPH_JULIA_RERANK_SIMILARITY_WEIGHT_ENV => Some("0.8".to_string()),
            _ => None,
        });

    assert_eq!(runtime.base_url.as_deref(), Some("http://127.0.0.1:8088"));
    assert_eq!(runtime.route.as_deref(), Some("/rerank"));
    assert_eq!(runtime.health_route.as_deref(), Some("/healthz"));
    assert_eq!(runtime.schema_version.as_deref(), Some("v1"));
    assert_eq!(runtime.timeout_secs, Some(15));
    assert_eq!(runtime.service_mode.as_deref(), Some("stream"));
    assert_eq!(
        runtime.analyzer_config_path.as_deref(),
        Some(DEFAULT_JULIA_ANALYZER_EXAMPLE_CONFIG_PATH)
    );
    assert_eq!(
        runtime.analyzer_strategy.as_deref(),
        Some("similarity_only")
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
      analyzer_strategy: "similarity_only"
      vector_weight: 0.2
      similarity_weight: 0.8
"#,
    )?;

    let runtime = LinkGraphJuliaRerankRuntimeConfig::resolve_with_env_lookup(&settings, |name| {
        Some(
            match name {
                LINK_GRAPH_JULIA_RERANK_BASE_URL_ENV => "http://127.0.0.1:9999",
                LINK_GRAPH_JULIA_RERANK_ROUTE_ENV => "/env-rerank",
                LINK_GRAPH_JULIA_RERANK_HEALTH_ROUTE_ENV => "/env-health",
                LINK_GRAPH_JULIA_RERANK_SCHEMA_VERSION_ENV => "v2",
                LINK_GRAPH_JULIA_RERANK_TIMEOUT_SECS_ENV => "77",
                LINK_GRAPH_JULIA_RERANK_SERVICE_MODE_ENV => "batch",
                LINK_GRAPH_JULIA_RERANK_ANALYZER_CONFIG_PATH_ENV => "config/env.toml",
                LINK_GRAPH_JULIA_RERANK_ANALYZER_STRATEGY_ENV => "linear_blend",
                LINK_GRAPH_JULIA_RERANK_VECTOR_WEIGHT_ENV => "0.7",
                LINK_GRAPH_JULIA_RERANK_SIMILARITY_WEIGHT_ENV => "0.3",
                _ => return None,
            }
            .to_string(),
        )
    });

    assert_eq!(runtime.base_url.as_deref(), Some("http://127.0.0.1:8088"));
    assert_eq!(runtime.route.as_deref(), Some("/rerank"));
    assert_eq!(runtime.health_route.as_deref(), Some("/healthz"));
    assert_eq!(runtime.schema_version.as_deref(), Some("v1"));
    assert_eq!(runtime.timeout_secs, Some(15));
    assert_eq!(runtime.service_mode.as_deref(), Some("stream"));
    assert_eq!(
        runtime.analyzer_strategy.as_deref(),
        Some("similarity_only")
    );
    assert_eq!(runtime.vector_weight, Some(0.2));
    assert_eq!(runtime.similarity_weight, Some(0.8));

    Ok(())
}

#[test]
fn julia_plugin_artifact_resolution_keeps_transport_diagnostics() {
    let selector = julia_deployment_artifact_selector();
    let runtime = LinkGraphJuliaRerankRuntimeConfig {
        base_url: Some("http://127.0.0.1:8088".to_string()),
        route: Some(DEFAULT_JULIA_RERANK_FLIGHT_ROUTE.to_string()),
        health_route: Some("/healthz".to_string()),
        schema_version: Some("v1".to_string()),
        timeout_secs: Some(15),
        service_mode: Some("stream".to_string()),
        analyzer_config_path: Some(DEFAULT_JULIA_ANALYZER_EXAMPLE_CONFIG_PATH.to_string()),
        analyzer_strategy: Some("similarity_only".to_string()),
        vector_weight: Some(0.2),
        similarity_weight: Some(0.8),
    };

    let Some(payload) = resolve_julia_plugin_artifact_payload_for_selector(&selector, &runtime)
    else {
        panic!("artifact payload");
    };

    assert_eq!(payload.plugin_id, selector.plugin_id);
    assert_eq!(payload.artifact_id, selector.artifact_id);
    assert_eq!(
        payload.selected_transport,
        Some(xiuxian_wendao_core::transport::PluginTransportKind::ArrowFlight)
    );
    assert_eq!(payload.fallback_from, None);
    assert_eq!(payload.fallback_reason, None);
}

#[test]
fn julia_plugin_artifact_rendering_serializes_resolved_payload()
-> Result<(), Box<dyn std::error::Error>> {
    let selector = julia_deployment_artifact_selector();
    let runtime = LinkGraphJuliaRerankRuntimeConfig {
        base_url: Some("http://127.0.0.1:8088".to_string()),
        route: Some(DEFAULT_JULIA_RERANK_FLIGHT_ROUTE.to_string()),
        health_route: Some("/healthz".to_string()),
        schema_version: Some("v1".to_string()),
        timeout_secs: Some(15),
        service_mode: Some("stream".to_string()),
        analyzer_config_path: Some(DEFAULT_JULIA_ANALYZER_EXAMPLE_CONFIG_PATH.to_string()),
        analyzer_strategy: Some("similarity_only".to_string()),
        vector_weight: Some(0.2),
        similarity_weight: Some(0.8),
    };

    let Some(rendered) = render_julia_plugin_artifact_toml_for_selector(&selector, &runtime)?
    else {
        panic!("rendered payload");
    };

    assert!(rendered.contains("plugin_id = \"xiuxian-wendao-julia\""));
    assert!(rendered.contains("artifact_id = \"deployment\""));
    assert!(rendered.contains("selected_transport = \"arrow_flight\""));

    Ok(())
}

#[test]
fn deployment_artifact_writes_toml_file() -> Result<(), Box<dyn std::error::Error>> {
    let artifact = LinkGraphJuliaDeploymentArtifact {
        artifact_schema_version: DEFAULT_JULIA_DEPLOYMENT_ARTIFACT_SCHEMA_VERSION.to_string(),
        generated_at: "2026-03-27T16:00:00+00:00".to_string(),
        base_url: Some("http://127.0.0.1:18080".to_string()),
        route: Some("/rerank".to_string()),
        health_route: Some("/health".to_string()),
        schema_version: Some("v1".to_string()),
        timeout_secs: Some(15),
        launch: LinkGraphJuliaAnalyzerLaunchManifest {
            launcher_path: DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH.to_string(),
            args: vec![
                "--service-mode".to_string(),
                "stream".to_string(),
                "--analyzer-strategy".to_string(),
                "similarity_only".to_string(),
            ],
        },
    };

    let temp = tempfile::tempdir()?;
    let artifact_path = temp
        .path()
        .join("nested")
        .join("julia_deployment_artifact.toml");
    artifact.write_toml_file(&artifact_path)?;

    let written = fs::read_to_string(&artifact_path)?;
    assert!(written.contains("artifact_schema_version = \"v1\""));
    assert!(written.contains("generated_at = \"2026-03-27T16:00:00+00:00\""));
    assert!(written.contains("base_url = \"http://127.0.0.1:18080\""));
    assert!(written.contains(&format!(
        "launcher_path = \"{DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH}\""
    )));
    assert!(written.contains("\"similarity_only\""));
    assert_eq!(written, artifact.to_toml_string()?);

    Ok(())
}

#[test]
fn deployment_artifact_writes_json_file() -> Result<(), Box<dyn std::error::Error>> {
    let artifact = LinkGraphJuliaDeploymentArtifact {
        artifact_schema_version: DEFAULT_JULIA_DEPLOYMENT_ARTIFACT_SCHEMA_VERSION.to_string(),
        generated_at: "2026-03-27T16:00:00+00:00".to_string(),
        base_url: Some("http://127.0.0.1:18080".to_string()),
        route: Some("/rerank".to_string()),
        health_route: Some("/health".to_string()),
        schema_version: Some("v1".to_string()),
        timeout_secs: Some(15),
        launch: LinkGraphJuliaAnalyzerLaunchManifest {
            launcher_path: DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH.to_string(),
            args: vec![
                "--service-mode".to_string(),
                "stream".to_string(),
                "--analyzer-strategy".to_string(),
                "similarity_only".to_string(),
            ],
        },
    };

    let temp = tempfile::tempdir()?;
    let artifact_path = temp
        .path()
        .join("nested")
        .join("julia_deployment_artifact.json");
    artifact.write_json_file(&artifact_path)?;

    let written = fs::read_to_string(&artifact_path)?;
    assert!(written.contains("\"artifact_schema_version\": \"v1\""));
    assert!(written.contains("\"generated_at\": \"2026-03-27T16:00:00+00:00\""));
    assert!(written.contains("\"base_url\": \"http://127.0.0.1:18080\""));
    assert!(written.contains(&format!(
        "\"launcher_path\": \"{DEFAULT_JULIA_ANALYZER_LAUNCHER_PATH}\""
    )));
    assert_eq!(written, artifact.to_json_string()?);

    Ok(())
}

#[test]
fn rerank_runtime_converts_into_generic_binding() {
    let binding = build_rerank_provider_binding(&LinkGraphJuliaRerankRuntimeConfig {
        base_url: Some("http://127.0.0.1:8088".to_string()),
        route: Some("/rerank".to_string()),
        health_route: Some("/healthz".to_string()),
        schema_version: Some("v2".to_string()),
        timeout_secs: Some(15),
        service_mode: Some("stream".to_string()),
        analyzer_config_path: Some("config/analyzer.toml".to_string()),
        analyzer_strategy: Some("linear_blend".to_string()),
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
    assert!(launch.args.iter().any(|value| value == "--service-mode"));
    assert!(launch.args.iter().any(|value| value == "linear_blend"));
}
