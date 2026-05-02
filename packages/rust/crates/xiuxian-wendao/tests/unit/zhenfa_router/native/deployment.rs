use super::{
    PluginArtifactSelector, WendaoPluginArtifactArgs, WendaoPluginArtifactOutputFormat,
    build_plugin_artifact_selector, export_plugin_artifact, render_plugin_artifact,
    render_plugin_artifact_json, render_plugin_artifact_toml,
};
use crate::set_link_graph_wendao_config_override;
use serial_test::serial;
use std::fs;
use xiuxian_wendao_builtin::{
    linked_builtin_julia_gateway_artifact_expected_json_fragments,
    linked_builtin_julia_gateway_artifact_expected_toml_fragments,
    linked_builtin_julia_gateway_artifact_path,
    linked_builtin_julia_gateway_artifact_runtime_config_toml,
};

fn tempdir_or_panic() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"))
}

fn write_config_and_set_override(temp: &tempfile::TempDir, body: &str) {
    let config_path = temp.path().join("wendao.toml");
    fs::write(&config_path, body).unwrap_or_else(|error| panic!("write config: {error}"));
    let config_path_string = config_path.to_string_lossy().to_string();
    set_link_graph_wendao_config_override(&config_path_string);
}

fn plugin_selector_or_panic() -> PluginArtifactSelector {
    let (plugin_id, artifact_id) = linked_builtin_julia_gateway_artifact_path();

    build_plugin_artifact_selector(&plugin_id, &artifact_id)
        .unwrap_or_else(|error| panic!("build plugin selector: {error}"))
}

#[test]
fn wendao_plugin_artifact_args_deserialize_selector_and_format() {
    let (plugin_id, artifact_id) = linked_builtin_julia_gateway_artifact_path();
    let args: WendaoPluginArtifactArgs = serde_json::from_value(serde_json::json!({
        "plugin_id": plugin_id.clone(),
        "artifact_id": artifact_id.clone(),
        "output_format": "json"
    }))
    .unwrap_or_else(|error| panic!("generic plugin-artifact args should deserialize: {error}"));

    assert_eq!(args.plugin_id, plugin_id);
    assert_eq!(args.artifact_id, artifact_id);
    assert!(matches!(
        args.output_format,
        WendaoPluginArtifactOutputFormat::Json
    ));
}

#[test]
fn wendao_plugin_artifact_args_default_to_toml_output() {
    let (plugin_id, artifact_id) = linked_builtin_julia_gateway_artifact_path();
    let args: WendaoPluginArtifactArgs = serde_json::from_value(serde_json::json!({
        "plugin_id": plugin_id.clone(),
        "artifact_id": artifact_id.clone()
    }))
    .unwrap_or_else(|error| panic!("generic plugin-artifact args should deserialize: {error}"));

    assert!(matches!(
        args.output_format,
        WendaoPluginArtifactOutputFormat::Toml
    ));
}

#[test]
fn wendao_plugin_artifact_args_deserialize_output_path() {
    let (plugin_id, artifact_id) = linked_builtin_julia_gateway_artifact_path();
    let args: WendaoPluginArtifactArgs = serde_json::from_value(serde_json::json!({
        "plugin_id": plugin_id.clone(),
        "artifact_id": artifact_id.clone(),
        "output_format": "json",
        "output_path": ".run/julia/artifact.json"
    }))
    .unwrap_or_else(|error| panic!("args with output_path should deserialize: {error}"));

    assert_eq!(
        args.output_path.as_deref(),
        Some(".run/julia/artifact.json")
    );
}

#[test]
#[serial]
fn render_plugin_artifact_toml_uses_runtime_config() {
    let temp = tempdir_or_panic();
    write_config_and_set_override(
        &temp,
        &linked_builtin_julia_gateway_artifact_runtime_config_toml(),
    );

    let selector = plugin_selector_or_panic();
    let rendered = render_plugin_artifact_toml(&selector)
        .unwrap_or_else(|error| panic!("render toml: {error}"));

    for fragment in linked_builtin_julia_gateway_artifact_expected_toml_fragments() {
        assert!(
            rendered.contains(&fragment),
            "expected rendered TOML to contain `{fragment}`: {rendered}"
        );
    }
    assert!(rendered.contains("generated_at = "));
    assert!(rendered.contains("selected_transport = \"arrow_flight\""));
}

#[test]
#[serial]
fn render_plugin_artifact_json_uses_runtime_config() {
    let temp = tempdir_or_panic();
    write_config_and_set_override(
        &temp,
        &linked_builtin_julia_gateway_artifact_runtime_config_toml(),
    );

    let selector = plugin_selector_or_panic();
    let rendered = render_plugin_artifact_json(&selector)
        .unwrap_or_else(|error| panic!("render json: {error}"));

    for fragment in linked_builtin_julia_gateway_artifact_expected_json_fragments() {
        assert!(
            rendered.contains(&fragment),
            "expected rendered JSON to contain `{fragment}`: {rendered}"
        );
    }
    assert!(rendered.contains("\"generated_at\": "));
}

#[test]
#[serial]
fn render_plugin_artifact_uses_selected_format() {
    let temp = tempdir_or_panic();
    write_config_and_set_override(
        &temp,
        &linked_builtin_julia_gateway_artifact_runtime_config_toml(),
    );

    let selector = plugin_selector_or_panic();
    let rendered = render_plugin_artifact(&selector, WendaoPluginArtifactOutputFormat::Json)
        .unwrap_or_else(|error| panic!("render generic plugin artifact: {error}"));

    for fragment in linked_builtin_julia_gateway_artifact_expected_json_fragments() {
        assert!(
            rendered.contains(&fragment),
            "expected rendered JSON to contain `{fragment}`: {rendered}"
        );
    }
}

#[test]
#[serial]
fn export_plugin_artifact_writes_json_file_when_requested() {
    let temp = tempdir_or_panic();
    let (plugin_id, artifact_id) = linked_builtin_julia_gateway_artifact_path();
    write_config_and_set_override(
        &temp,
        &linked_builtin_julia_gateway_artifact_runtime_config_toml(),
    );

    let output_path = temp.path().join("exports").join("plugin-artifact.json");
    let message = export_plugin_artifact(&WendaoPluginArtifactArgs {
        plugin_id: plugin_id.clone(),
        artifact_id: artifact_id.clone(),
        output_format: WendaoPluginArtifactOutputFormat::Json,
        output_path: Some(output_path.to_string_lossy().to_string()),
    })
    .unwrap_or_else(|error| panic!("export generic plugin artifact: {error}"));

    assert!(message.contains("Wrote plugin artifact"));
    assert!(message.contains(&plugin_id));
    assert!(message.contains(&artifact_id));
    let written = fs::read_to_string(&output_path)
        .unwrap_or_else(|error| panic!("read written json: {error}"));

    for fragment in linked_builtin_julia_gateway_artifact_expected_json_fragments() {
        assert!(
            written.contains(&fragment),
            "expected written JSON to contain `{fragment}`: {written}"
        );
    }
}
