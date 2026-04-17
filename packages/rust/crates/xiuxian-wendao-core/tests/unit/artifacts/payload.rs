use crate::artifacts::PluginLaunchSpec;
use crate::capabilities::ContractVersion;
use crate::ids::{ArtifactId, PluginId};
use crate::transport::{PluginTransportEndpoint, PluginTransportKind};

use super::PluginArtifactPayload;

#[test]
fn artifact_payload_serializes_to_toml_and_json() -> Result<(), Box<dyn std::error::Error>> {
    let payload = PluginArtifactPayload {
        plugin_id: PluginId("wendao-julia".to_string()),
        artifact_id: ArtifactId("deployment".to_string()),
        artifact_schema_version: ContractVersion("v1".to_string()),
        generated_at: "2026-03-28T12:00:00Z".to_string(),
        endpoint: Some(PluginTransportEndpoint {
            base_url: Some("http://127.0.0.1:8815".to_string()),
            route: Some("/rerank".to_string()),
            health_route: Some("/healthz".to_string()),
            timeout_secs: Some(30),
            max_in_flight_requests: None,
        }),
        schema_version: Some("v1".to_string()),
        launch: Some(PluginLaunchSpec {
            launcher_path: ".data/WendaoAnalyzer/scripts/run.sh".to_string(),
            args: vec!["--stdio".to_string()],
        }),
        selected_transport: Some(PluginTransportKind::ArrowFlight),
        fallback_from: None,
        fallback_reason: None,
    };

    let toml = payload.to_toml_string()?;
    let json = payload.to_json_string()?;

    assert!(toml.contains("wendao-julia"));
    assert!(toml.contains("selected_transport = \"arrow_flight\""));
    assert!(toml.contains("route = \"/rerank\""));
    assert!(json.contains("\"deployment\""));
    assert!(!json.contains("\"fallback_reason\""));

    Ok(())
}
