use schemars::JsonSchema;
use serde::Deserialize;
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError, zhenfa_tool};

use super::shared::{require_non_empty_argument, serialize_payload};
use crate::analyzers::projection::ProjectionPageKind;
use crate::zhenfa_router::native::resolve_docs_tool_runtime;

/// Arguments for the `wendao.docs.search_document_structure` native tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WendaoDocsSearchDocumentStructureArgs {
    /// User-provided docs/page-index search string.
    query: String,
    /// Optional projected-page family filter.
    kind: Option<ProjectionPageKind>,
    /// Maximum number of page-index node hits to return.
    limit: Option<usize>,
}

/// Search docs-facing page-index nodes and return serialized candidate hits.
///
/// # Errors
///
/// Returns a [`ZhenfaError`] when arguments are invalid, the docs capability
/// service is missing from the native context, or the underlying docs lookup
/// fails.
#[allow(missing_docs)]
#[zhenfa_tool(
    name = "wendao.docs.search_document_structure",
    description = "Search docs-facing page-index nodes and return serialized candidate hits.",
    tool_struct = "WendaoDocsSearchDocumentStructureTool"
)]
pub fn wendao_docs_search_document_structure(
    ctx: &ZhenfaContext,
    args: WendaoDocsSearchDocumentStructureArgs,
) -> Result<String, ZhenfaError> {
    let WendaoDocsSearchDocumentStructureArgs { query, kind, limit } = args;
    let query = require_non_empty_argument(&query, "query")?;
    let runtime = resolve_docs_tool_runtime(ctx)?;
    let result = runtime
        .search_document_structure(&query, kind, limit.unwrap_or(10).max(1))
        .map_err(|error| ZhenfaError::execution(error.to_string()))?;
    serialize_payload(&result)
}
