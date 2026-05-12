//! `zhenfa_router::native::docs::page_index_outline` owns Wendao native docs page index outline behavior.

use schemars::JsonSchema;
use serde::Deserialize;
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError, zhenfa_tool};

use super::shared::{require_non_empty_argument, serialize_payload};
use crate::zhenfa_router::native::resolve_docs_tool_runtime;

/// Arguments for the `wendao.docs.get_page_index_outline` native tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WendaoDocsGetPageIndexOutlineArgs {
    /// Stable docs-facing page identifier.
    page_id: String,
}

/// Resolve one text-free docs-facing page-index tree and return its payload.
///
/// # Errors
///
/// Returns a [`ZhenfaError`] when arguments are invalid, the docs capability
/// service is missing from the native context, or the underlying docs lookup
/// fails.
#[allow(missing_docs)]
#[zhenfa_tool(
    name = "wendao.docs.get_page_index_outline",
    description = "Open one text-free docs-facing page-index tree and return its serialized payload.",
    tool_struct = "WendaoDocsGetPageIndexOutlineTool"
)]
pub fn wendao_docs_get_page_index_outline(
    ctx: &ZhenfaContext,
    args: WendaoDocsGetPageIndexOutlineArgs,
) -> Result<String, ZhenfaError> {
    let WendaoDocsGetPageIndexOutlineArgs { page_id } = args;
    let page_id = require_non_empty_argument(&page_id, "page_id")?;
    let runtime = resolve_docs_tool_runtime(ctx)?;
    let result = runtime
        .get_page_index_outline(&page_id)
        .map_err(|error| ZhenfaError::execution(error.to_string()))?;
    serialize_payload(&result)
}
