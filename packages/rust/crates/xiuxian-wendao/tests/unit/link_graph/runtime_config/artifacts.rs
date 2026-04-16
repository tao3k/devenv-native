#[cfg(all(feature = "builtin-plugins", feature = "julia"))]
use crate::link_graph::runtime_config::{
    render_link_graph_plugin_artifact_toml_for_selector,
    resolve_link_graph_plugin_artifact_for_selector,
};
#[cfg(all(feature = "builtin-plugins", feature = "julia"))]
use crate::link_graph::set_link_graph_wendao_config_override;
#[cfg(all(feature = "builtin-plugins", feature = "julia"))]
use serial_test::serial;
#[cfg(all(feature = "builtin-plugins", feature = "julia"))]
use std::fs;
#[cfg(all(feature = "builtin-plugins", feature = "julia"))]
use xiuxian_wendao_builtin::linked_builtin_julia_deployment_artifact_selector;

#[cfg(all(feature = "builtin-plugins", feature = "julia"))]
#[test]
#[serial]
fn resolve_plugin_artifact_resolves_julia_deployment_payload()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        r#"[link_graph.retrieval.julia_rerank]
base_url = "http://127.0.0.1:8088"
route = "/rerank"
health_route = "/healthz"
schema_version = "v1"
timeout_secs = 15
service_mode = "stream"
"#,
    )?;
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());

    let selector = linked_builtin_julia_deployment_artifact_selector();
    let Some(artifact) = resolve_link_graph_plugin_artifact_for_selector(&selector) else {
        panic!("artifact");
    };
    assert_eq!(artifact.plugin_id, selector.plugin_id);
    assert_eq!(artifact.artifact_id, selector.artifact_id);
    assert_eq!(artifact.artifact_schema_version.0, "v1");
    assert_eq!(
        artifact
            .endpoint
            .as_ref()
            .and_then(|endpoint| endpoint.base_url.as_deref()),
        Some("http://127.0.0.1:8088")
    );
    assert_eq!(
        artifact.selected_transport,
        Some(xiuxian_wendao_core::transport::PluginTransportKind::ArrowFlight)
    );
    assert_eq!(artifact.fallback_from, None);
    assert_eq!(artifact.fallback_reason, None);

    Ok(())
}

#[cfg(all(feature = "builtin-plugins", feature = "julia"))]
#[test]
#[serial]
fn render_plugin_artifact_toml_renders_julia_deployment_payload()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        r#"[link_graph.retrieval.julia_rerank]
base_url = "http://127.0.0.1:8088"
route = "/rerank"
schema_version = "v1"
"#,
    )?;
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());

    let Some(rendered) = render_link_graph_plugin_artifact_toml_for_selector(
        &linked_builtin_julia_deployment_artifact_selector(),
    )?
    else {
        panic!("rendered artifact");
    };
    assert!(rendered.contains("plugin_id = \"xiuxian-wendao-julia\""));
    assert!(rendered.contains("artifact_id = \"deployment\""));
    assert!(rendered.contains("route = \"/rerank\""));
    assert!(rendered.contains("selected_transport = \"arrow_flight\""));

    Ok(())
}
