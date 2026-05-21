//! `zhenfa_router::native::deployment` owns Wendao zhenfa router native deployment behavior.

use schemars::JsonSchema;
use serde::Deserialize;
use std::path::Path;
use xiuxian_wendao_core::{
    artifacts::PluginArtifactSelector,
    ids::{ArtifactId, PluginId},
};
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError, zhenfa_tool};

use crate::link_graph::runtime_config::{
    render_link_graph_plugin_artifact_toml_for_selector,
    resolve_link_graph_plugin_artifact_for_selector,
};

/// Output formats for visible plugin-artifact export.
#[derive(Debug, Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WendaoPluginArtifactOutputFormat {
    /// Render the plugin artifact as TOML.
    #[default]
    Toml,
    /// Render the plugin artifact as structured JSON.
    Json,
}

/// Arguments for exporting one resolved plugin artifact.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct WendaoPluginArtifactArgs {
    /// Plugin identifier that owns the artifact.
    pub plugin_id: String,
    /// Artifact identifier within the selected plugin.
    pub artifact_id: String,
    /// Optional output format. Defaults to TOML.
    #[serde(default)]
    pub output_format: WendaoPluginArtifactOutputFormat,
    /// Optional destination path for persisting the rendered artifact.
    #[serde(default)]
    pub output_path: Option<String>,
}

impl WendaoPluginArtifactArgs {
    fn selector(&self) -> Result<PluginArtifactSelector, ZhenfaError> {
        build_plugin_artifact_selector(self.plugin_id.trim(), self.artifact_id.trim())
    }
}

fn build_plugin_artifact_selector(
    plugin_id: &str,
    artifact_id: &str,
) -> Result<PluginArtifactSelector, ZhenfaError> {
    if plugin_id.is_empty() {
        return Err(ZhenfaError::execution(
            "export plugin artifact: `plugin_id` must be non-empty",
        ));
    }
    if artifact_id.is_empty() {
        return Err(ZhenfaError::execution(
            "export plugin artifact: `artifact_id` must be non-empty",
        ));
    }

    Ok(PluginArtifactSelector {
        plugin_id: PluginId(plugin_id.to_string()),
        artifact_id: ArtifactId(artifact_id.to_string()),
    })
}

/// Export one resolved plugin artifact.
///
/// # Errors
///
/// Returns a [`ZhenfaError`] when the selected plugin artifact cannot be
/// serialized into the requested format.
#[allow(missing_docs)]
#[allow(clippy::needless_pass_by_value)]
#[zhenfa_tool(
    name = "wendao.plugin_artifact",
    description = "Export one resolved plugin artifact selected by plugin_id and artifact_id as TOML or structured JSON.",
    tool_struct = "WendaoPluginArtifactTool"
)]
pub fn wendao_plugin_artifact(
    _ctx: &ZhenfaContext,
    args: WendaoPluginArtifactArgs,
) -> Result<String, ZhenfaError> {
    export_plugin_artifact(&args)
}

/// Render the resolved plugin artifact as TOML.
///
/// # Errors
///
/// Returns a [`ZhenfaError`] when TOML serialization fails.
pub fn render_plugin_artifact_toml(
    selector: &PluginArtifactSelector,
) -> Result<String, ZhenfaError> {
    render_link_graph_plugin_artifact_toml_for_selector(selector)
        .map_err(|error| ZhenfaError::execution(format!("export plugin artifact: {error}")))?
        .ok_or_else(|| ZhenfaError::execution("export plugin artifact: not found"))
}

/// Render the resolved plugin artifact as structured JSON.
///
/// # Errors
///
/// Returns a [`ZhenfaError`] when JSON serialization fails.
pub fn render_plugin_artifact_json(
    selector: &PluginArtifactSelector,
) -> Result<String, ZhenfaError> {
    resolve_link_graph_plugin_artifact_for_selector(selector)
        .ok_or_else(|| ZhenfaError::execution("export plugin artifact as json: not found"))?
        .to_json_string()
        .map_err(|error| ZhenfaError::execution(format!("export plugin artifact as json: {error}")))
}

/// Render the resolved plugin artifact using the selected format.
///
/// # Errors
///
/// Returns a [`ZhenfaError`] when serialization fails.
pub fn render_plugin_artifact(
    selector: &PluginArtifactSelector,
    output_format: WendaoPluginArtifactOutputFormat,
) -> Result<String, ZhenfaError> {
    match output_format {
        WendaoPluginArtifactOutputFormat::Toml => render_plugin_artifact_toml(selector),
        WendaoPluginArtifactOutputFormat::Json => render_plugin_artifact_json(selector),
    }
}

/// Export one resolved plugin artifact, optionally writing it to a file.
///
/// # Errors
///
/// Returns a [`ZhenfaError`] when serialization or file writing fails.
pub fn export_plugin_artifact(args: &WendaoPluginArtifactArgs) -> Result<String, ZhenfaError> {
    let selector = args.selector()?;

    if let Some(output_path) = args.output_path.as_deref() {
        let artifact = resolve_link_graph_plugin_artifact_for_selector(&selector)
            .ok_or_else(|| ZhenfaError::execution("write plugin artifact: artifact not found"))?;
        let path = Path::new(output_path);
        match args.output_format {
            WendaoPluginArtifactOutputFormat::Toml => artifact.write_toml_file(path),
            WendaoPluginArtifactOutputFormat::Json => artifact.write_json_file(path),
        }
        .map_err(|error| {
            ZhenfaError::execution(format!(
                "write plugin artifact {} / {} to {}: {error}",
                selector.plugin_id.0,
                selector.artifact_id.0,
                path.display()
            ))
        })?;

        return Ok(format!(
            "Wrote plugin artifact {} / {} ({}) to {}",
            selector.plugin_id.0,
            selector.artifact_id.0,
            match args.output_format {
                WendaoPluginArtifactOutputFormat::Toml => "toml",
                WendaoPluginArtifactOutputFormat::Json => "json",
            },
            path.display()
        ));
    }

    render_plugin_artifact(&selector, args.output_format)
}

#[cfg(all(test, feature = "julia"))]
#[path = "../../../tests/unit/zhenfa_router/native/deployment.rs"]
mod tests;
