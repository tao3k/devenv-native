use schemars::JsonSchema;
use serde::Deserialize;
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError, zhenfa_tool};

use super::shared::{require_non_empty_argument, serialize_payload};
use crate::zhenfa_router::native::resolve_docs_tool_runtime;

/// Arguments for the `wendao.docs.get_page_index_tree` native tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WendaoDocsGetPageIndexTreeArgs {
    /// Stable docs-facing page identifier.
    page_id: String,
}

/// Resolve one docs-facing page-index tree and return its serialized payload.
///
/// # Errors
///
/// Returns a [`ZhenfaError`] when arguments are invalid, the docs capability
/// service is missing from the native context, or the underlying docs lookup
/// fails.
#[allow(missing_docs)]
#[zhenfa_tool(
    name = "wendao.docs.get_page_index_tree",
    description = "Open one docs-facing page-index tree and return its serialized payload.",
    tool_struct = "WendaoDocsGetPageIndexTreeTool"
)]
pub fn wendao_docs_get_page_index_tree(
    ctx: &ZhenfaContext,
    args: WendaoDocsGetPageIndexTreeArgs,
) -> Result<String, ZhenfaError> {
    let WendaoDocsGetPageIndexTreeArgs { page_id } = args;
    let page_id = require_non_empty_argument(&page_id, "page_id")?;
    let runtime = resolve_docs_tool_runtime(ctx)?;
    let result = runtime
        .get_page_index_tree(&page_id)
        .map_err(|error| ZhenfaError::execution(error.to_string()))?;
    serialize_payload(&result)
}
