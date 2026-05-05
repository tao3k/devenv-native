//! JSON-RPC helpers for Qianhuan Zhenfa render and reload actions.

use std::sync::Arc;

use anyhow::Context;
use serde_json::{Value, json};
use xiuxian_zhenfa::{INTERNAL_ERROR_CODE, JsonRpcErrorObject};

use crate::manifestation::{ManifestationManager, ManifestationRenderRequest};

/// Execute `qianhuan.render` from JSON-RPC parameters.
///
/// # Errors
/// Returns JSON-RPC errors when params are invalid or rendering fails.
pub fn render_from_rpc_params(
    manager: &Arc<ManifestationManager>,
    params: Value,
) -> Result<String, JsonRpcErrorObject> {
    let request: ManifestationRenderRequest = serde_json::from_value(params).map_err(|error| {
        JsonRpcErrorObject::invalid_params(format!("invalid qianhuan.render params: {error}"))
    })?;
    render(manager, &request).map_err(|error| internal_error(&error))
}

/// Execute `qianhuan.reload` and return a lean payload string.
///
/// # Errors
/// Returns JSON-RPC internal error when reload execution fails.
pub fn reload_for_rpc(manager: &Arc<ManifestationManager>) -> Result<String, JsonRpcErrorObject> {
    let reloaded = manager
        .reload_templates_if_changed()
        .context("failed to reload manifestation templates")
        .map_err(|error| internal_error(&error))?;
    Ok(format!("<qianhuan_reload reloaded=\"{reloaded}\" />"))
}

/// Render one manifestation template request.
///
/// # Errors
/// Returns an error when manifestation rendering fails.
pub(crate) fn render(
    manager: &Arc<ManifestationManager>,
    request: &ManifestationRenderRequest,
) -> anyhow::Result<String> {
    manager.render_request(request).with_context(|| {
        format!(
            "failed to render template `{}`",
            request.target.template_name()
        )
    })
}

fn internal_error(error: &anyhow::Error) -> JsonRpcErrorObject {
    JsonRpcErrorObject::new(
        INTERNAL_ERROR_CODE,
        "qianhuan operation failed",
        Some(json!({ "details": error.to_string() })),
    )
}
