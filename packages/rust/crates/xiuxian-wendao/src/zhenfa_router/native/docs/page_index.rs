use schemars::JsonSchema;
use serde::Deserialize;
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError, zhenfa_tool};

use super::shared::serialize_payload;
use crate::zhenfa_router::native::resolve_docs_tool_runtime;

/// Arguments for the `wendao.docs.get_page_index` native tool.
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct WendaoDocsGetPageIndexArgs {}

/// Resolve one repo-scoped text-free docs-facing page-index tree catalog and
/// return its serialized payload.
///
/// # Errors
///
/// Returns a [`ZhenfaError`] when the docs capability service is missing from
/// the native context or the underlying docs lookup fails.
#[allow(missing_docs)]
#[zhenfa_tool(
    name = "wendao.docs.get_page_index",
    description = "Open one repo-scoped text-free docs-facing page-index tree catalog and return its serialized payload.",
    tool_struct = "WendaoDocsGetPageIndexTool"
)]
pub fn wendao_docs_get_page_index(
    ctx: &ZhenfaContext,
    _args: WendaoDocsGetPageIndexArgs,
) -> Result<String, ZhenfaError> {
    let runtime = resolve_docs_tool_runtime(ctx)?;
    let result = runtime
        .get_page_index()
        .map_err(|error| ZhenfaError::execution(error.to_string()))?;
    serialize_payload(&result)
}
