use serde_json::{Value, json};

use super::{
    DEFAULT_JULIA_DEPLOYMENT_ARTIFACT_SCHEMA_VERSION, DEFAULT_JULIA_RERANK_FLIGHT_ROUTE,
    DEFAULT_JULIA_SEARCH_LAUNCHER_PATH, JULIA_DEPLOYMENT_ARTIFACT_ID, JULIA_PLUGIN_ID,
    LinkGraphJuliaDeploymentArtifact, LinkGraphJuliaSearchLaunchManifest,
};

const OPENAPI_EXAMPLE_BASE_URL: &str = "http://127.0.0.1:18080";
const OPENAPI_EXAMPLE_GENERATED_AT: &str = "2026-03-27T16:00:00+00:00";
const OPENAPI_EXAMPLE_HEALTH_ROUTE: &str = "/healthz";
const OPENAPI_EXAMPLE_SCHEMA_VERSION: &str = "v1";
const OPENAPI_EXAMPLE_TIMEOUT_SECS: u64 = 30;
const OPENAPI_EXAMPLE_SERVICE_MODE: &str = "stream";

/// Return the curated `OpenAPI` JSON example for the generic Julia deployment artifact.
#[must_use]
pub fn julia_plugin_artifact_openapi_json_example() -> Value {
    json!({
        "pluginId": JULIA_PLUGIN_ID,
        "artifactId": JULIA_DEPLOYMENT_ARTIFACT_ID,
        "schemaVersion": OPENAPI_EXAMPLE_SCHEMA_VERSION,
        "baseUrl": OPENAPI_EXAMPLE_BASE_URL,
        "route": DEFAULT_JULIA_RERANK_FLIGHT_ROUTE,
    })
}

/// Return the curated `OpenAPI` TOML example for the generic Julia deployment artifact.
#[must_use]
pub fn julia_plugin_artifact_openapi_toml_example() -> String {
    format!(
        "plugin_id = \"{JULIA_PLUGIN_ID}\"\nartifact_id = \"{JULIA_DEPLOYMENT_ARTIFACT_ID}\"\nschema_version = \"{OPENAPI_EXAMPLE_SCHEMA_VERSION}\"\nbase_url = \"{OPENAPI_EXAMPLE_BASE_URL}\"\nroute = \"{DEFAULT_JULIA_RERANK_FLIGHT_ROUTE}\"\n"
    )
}

/// Return the curated Julia deployment artifact example used by the bundled `OpenAPI` contract.
#[must_use]
pub fn julia_deployment_artifact_openapi_example() -> LinkGraphJuliaDeploymentArtifact {
    LinkGraphJuliaDeploymentArtifact {
        artifact_schema_version: DEFAULT_JULIA_DEPLOYMENT_ARTIFACT_SCHEMA_VERSION.to_string(),
        generated_at: OPENAPI_EXAMPLE_GENERATED_AT.to_string(),
        base_url: Some(OPENAPI_EXAMPLE_BASE_URL.to_string()),
        route: Some(DEFAULT_JULIA_RERANK_FLIGHT_ROUTE.to_string()),
        health_route: Some(OPENAPI_EXAMPLE_HEALTH_ROUTE.to_string()),
        schema_version: Some(OPENAPI_EXAMPLE_SCHEMA_VERSION.to_string()),
        timeout_secs: Some(OPENAPI_EXAMPLE_TIMEOUT_SECS),
        launch: LinkGraphJuliaSearchLaunchManifest {
            launcher_path: DEFAULT_JULIA_SEARCH_LAUNCHER_PATH.to_string(),
            args: vec![
                "--mode".to_string(),
                OPENAPI_EXAMPLE_SERVICE_MODE.to_string(),
            ],
        },
    }
}

/// Return the curated `OpenAPI` JSON example for the legacy Julia deployment artifact path.
#[must_use]
pub fn julia_deployment_artifact_openapi_json_example() -> Value {
    json!({
        "artifactSchemaVersion": DEFAULT_JULIA_DEPLOYMENT_ARTIFACT_SCHEMA_VERSION,
        "generatedAt": OPENAPI_EXAMPLE_GENERATED_AT,
        "baseUrl": OPENAPI_EXAMPLE_BASE_URL,
        "route": DEFAULT_JULIA_RERANK_FLIGHT_ROUTE,
        "healthRoute": OPENAPI_EXAMPLE_HEALTH_ROUTE,
        "schemaVersion": OPENAPI_EXAMPLE_SCHEMA_VERSION,
        "timeoutSecs": OPENAPI_EXAMPLE_TIMEOUT_SECS,
        "launch": {
            "launcherPath": DEFAULT_JULIA_SEARCH_LAUNCHER_PATH,
            "args": [
                "--mode",
                OPENAPI_EXAMPLE_SERVICE_MODE,
            ],
        },
    })
}

/// Return the curated `OpenAPI` TOML example for the legacy Julia deployment artifact path.
///
/// # Errors
///
/// Returns an error when the curated example cannot be serialized into TOML.
pub fn julia_deployment_artifact_openapi_toml_example() -> Result<String, toml::ser::Error> {
    julia_deployment_artifact_openapi_example().to_toml_string()
}
