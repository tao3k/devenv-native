//! Native Zhenfa tool adapters for Qianhuan render and reload actions.

use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use xiuxian_wendao::skill_runtime::SkillRuntimeResolver;
use xiuxian_wendao_core::WendaoResourceUri;
use xiuxian_zhenfa::{
    INVALID_PARAMS_CODE, JsonRpcErrorObject, ZhenfaContext, ZhenfaError, zhenfa_tool,
};

use crate::manifestation::{
    ManifestationManager, ManifestationRenderRequest, ManifestationRuntimeContext,
    ManifestationTemplateTarget, MemoryTemplateRecord,
};

use super::rpc::{reload_for_rpc, render_from_rpc_params};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub(crate) struct QianhuanRenderArgs {
    target: String,
    data: Value,
    #[serde(default)]
    runtime: ManifestationRuntimeContext,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub(crate) struct QianhuanReloadArgs {}

/// Render one Qianhuan template through native zhenfa dispatch.
#[zhenfa_tool(
    name = "qianhuan.render",
    description = "Render a Qianhuan template through native zhenfa dispatch.",
    tool_struct = "QianhuanRenderTool"
)]
pub fn qianhuan_render(
    ctx: &ZhenfaContext,
    args: QianhuanRenderArgs,
) -> Result<String, ZhenfaError> {
    let manager = resolve_manager(ctx)?;
    let request = build_render_request(args)?;
    hydrate_semantic_template_target(ctx, &manager, &request)?;
    let params = serde_json::to_value(request).map_err(|error| {
        ZhenfaError::invalid_arguments(format!("invalid qianhuan.render params: {error}"))
    })?;
    render_from_rpc_params(&manager, params).map_err(map_jsonrpc_error)
}

/// Reload Qianhuan templates through native zhenfa dispatch.
#[zhenfa_tool(
    name = "qianhuan.reload",
    description = "Reload Qianhuan templates through native zhenfa dispatch.",
    tool_struct = "QianhuanReloadTool",
    mutation_scope = "qianhuan.reload.templates"
)]
pub fn qianhuan_reload(
    ctx: &ZhenfaContext,
    _args: QianhuanReloadArgs,
) -> Result<String, ZhenfaError> {
    let manager = resolve_manager(ctx)?;
    reload_for_rpc(&manager).map_err(map_jsonrpc_error)
}

fn resolve_manager(
    ctx: &ZhenfaContext,
) -> Result<std::sync::Arc<ManifestationManager>, ZhenfaError> {
    ctx.get_extension::<ManifestationManager>().ok_or_else(|| {
        ZhenfaError::execution("missing ManifestationManager in zhenfa context extensions")
    })
}

fn resolve_skill_runtime(
    ctx: &ZhenfaContext,
) -> Result<std::sync::Arc<SkillRuntimeResolver>, ZhenfaError> {
    ctx.get_extension::<SkillRuntimeResolver>().ok_or_else(|| {
        ZhenfaError::execution("missing SkillRuntimeResolver in zhenfa context extensions")
    })
}

fn build_render_request(
    args: QianhuanRenderArgs,
) -> Result<ManifestationRenderRequest, ZhenfaError> {
    let target = parse_render_target(args.target.as_str())?;
    Ok(ManifestationRenderRequest {
        target,
        data: args.data,
        runtime: args.runtime,
    })
}

fn parse_render_target(raw: &str) -> Result<ManifestationTemplateTarget, ZhenfaError> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err(ZhenfaError::invalid_arguments(
            "`target` must be a non-empty string",
        ));
    }
    let target = match normalized {
        "daily_agenda" => ManifestationTemplateTarget::DailyAgenda,
        "system_prompt_v2_xml" => ManifestationTemplateTarget::SystemPromptV2Xml,
        other => ManifestationTemplateTarget::Custom(other.to_string()),
    };
    Ok(target)
}

fn hydrate_semantic_template_target(
    ctx: &ZhenfaContext,
    manager: &std::sync::Arc<ManifestationManager>,
    request: &ManifestationRenderRequest,
) -> Result<(), ZhenfaError> {
    let ManifestationTemplateTarget::Custom(target) = &request.target else {
        return Ok(());
    };
    if !target.trim().to_ascii_lowercase().starts_with("wendao://") {
        return Ok(());
    }

    let normalized_target = target.trim();
    WendaoResourceUri::parse(normalized_target).map_err(|error| {
        ZhenfaError::invalid_arguments(format!(
            "invalid semantic template target `{normalized_target}`: {error}"
        ))
    })?;
    let resolver = resolve_skill_runtime(ctx)?;
    let content = resolver.read_utf8(normalized_target).map_err(|error| {
        ZhenfaError::execution(format!(
            "failed to resolve semantic template target `{normalized_target}`: {error}"
        ))
    })?;
    manager
        .upsert_template_from_memory(MemoryTemplateRecord::new(normalized_target, None, content))
        .map_err(|error| {
            ZhenfaError::execution(format!(
                "failed to upsert semantic template target `{normalized_target}`: {error}"
            ))
        })?;
    Ok(())
}

fn map_jsonrpc_error(error: JsonRpcErrorObject) -> ZhenfaError {
    let details = error
        .data
        .as_ref()
        .and_then(|value| value.get("details"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let message = if let Some(details) = details {
        format!("{}: {details}", error.message)
    } else {
        error.message
    };
    if error.code == INVALID_PARAMS_CODE {
        ZhenfaError::invalid_arguments(message)
    } else {
        ZhenfaError::execution_with_code(error.code, message)
    }
}
