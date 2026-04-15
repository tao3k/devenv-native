use schemars::JsonSchema;
use serde::Deserialize;
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError, zhenfa_tool};

use super::shared::{require_non_empty_argument, serialize_payload};
use crate::zhenfa_router::native::resolve_docs_tool_runtime;

/// Arguments for the `wendao.docs.get_document_structure` native tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WendaoDocsGetDocumentStructureArgs {
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
    name = "wendao.docs.get_document_structure",
    description = "Open one docs-facing page-index tree and return its serialized payload.",
    tool_struct = "WendaoDocsGetDocumentStructureTool"
)]
pub fn wendao_docs_get_document_structure(
    ctx: &ZhenfaContext,
    args: WendaoDocsGetDocumentStructureArgs,
) -> Result<String, ZhenfaError> {
    let WendaoDocsGetDocumentStructureArgs { page_id } = args;
    let page_id = require_non_empty_argument(&page_id, "page_id")?;
    let runtime = resolve_docs_tool_runtime(ctx)?;
    let result = runtime
        .get_document_structure(&page_id)
        .map_err(|error| ZhenfaError::execution(error.to_string()))?;
    serialize_payload(&result)
}
