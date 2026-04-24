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
        launch: LinkGraphJuliaSearchLaunchManifest {
            launcher_path: DEFAULT_JULIA_SEARCH_LAUNCHER_PATH.to_string(),
            args: vec!["--mode".to_string(), "stream".to_string()],
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
            launcher_path: DEFAULT_JULIA_SEARCH_LAUNCHER_PATH.to_string(),
            args: vec!["--mode".to_string(), "stream".to_string()],
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
fn deployment_artifact_writes_toml_file() -> Result<(), Box<dyn std::error::Error>> {
    let artifact = LinkGraphJuliaDeploymentArtifact {
        artifact_schema_version: DEFAULT_JULIA_DEPLOYMENT_ARTIFACT_SCHEMA_VERSION.to_string(),
        generated_at: "2026-03-27T16:00:00+00:00".to_string(),
        base_url: Some("http://127.0.0.1:18080".to_string()),
        route: Some("/rerank".to_string()),
        health_route: Some("/health".to_string()),
        schema_version: Some("v1".to_string()),
        timeout_secs: Some(15),
        launch: LinkGraphJuliaSearchLaunchManifest {
            launcher_path: DEFAULT_JULIA_SEARCH_LAUNCHER_PATH.to_string(),
            args: vec!["--mode".to_string(), "stream".to_string()],
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
        "launcher_path = \"{DEFAULT_JULIA_SEARCH_LAUNCHER_PATH}\""
    )));
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
        launch: LinkGraphJuliaSearchLaunchManifest {
            launcher_path: DEFAULT_JULIA_SEARCH_LAUNCHER_PATH.to_string(),
            args: vec!["--mode".to_string(), "stream".to_string()],
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
        "\"launcher_path\": \"{DEFAULT_JULIA_SEARCH_LAUNCHER_PATH}\""
    )));
    assert_eq!(written, artifact.to_json_string()?);

    Ok(())
}
