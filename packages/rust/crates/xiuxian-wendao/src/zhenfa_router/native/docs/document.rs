use schemars::JsonSchema;
use serde::Deserialize;
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError, zhenfa_tool};

use super::shared::{require_non_empty_argument, serialize_payload};
use crate::zhenfa_router::native::resolve_docs_tool_runtime;

/// Arguments for the `wendao.docs.get_document` native tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WendaoDocsGetDocumentArgs {
    /// Stable docs-facing page identifier.
    page_id: String,
}

/// Resolve one docs-facing document page and return its serialized payload.
///
/// # Errors
///
/// Returns a [`ZhenfaError`] when arguments are invalid, the docs capability
/// service is missing from the native context, or the underlying docs lookup
/// fails.
#[allow(missing_docs)]
#[zhenfa_tool(
    name = "wendao.docs.get_document",
    description = "Open one docs-facing projected page and return its serialized payload.",
    tool_struct = "WendaoDocsGetDocumentTool"
)]
pub fn wendao_docs_get_document(
    ctx: &ZhenfaContext,
    args: WendaoDocsGetDocumentArgs,
) -> Result<String, ZhenfaError> {
    let WendaoDocsGetDocumentArgs { page_id } = args;
    let page_id = require_non_empty_argument(&page_id, "page_id")?;
    let runtime = resolve_docs_tool_runtime(ctx)?;
    let result = runtime
        .get_document(&page_id)
        .map_err(|error| ZhenfaError::execution(error.to_string()))?;
    serialize_payload(&result)
}
