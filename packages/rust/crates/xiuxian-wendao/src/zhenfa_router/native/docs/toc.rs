//! `zhenfa_router::native::docs::toc` owns Wendao native docs toc behavior.

use schemars::JsonSchema;
use serde::Deserialize;
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError, zhenfa_tool};

use super::shared::serialize_payload;
use crate::zhenfa_router::native::resolve_docs_tool_runtime;

/// Arguments for the `wendao.docs.get_toc_documents` native tool.
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct WendaoDocsGetTocDocumentsArgs {}

/// Resolve repository-scoped docs markdown TOC/page-index documents and return
/// their serialized payload.
///
/// # Errors
///
/// Returns a [`ZhenfaError`] when the docs capability service is missing from
/// the native context or the underlying docs lookup fails.
#[allow(missing_docs)]
#[zhenfa_tool(
    name = "wendao.docs.get_toc_documents",
    description = "Open repository-scoped docs markdown TOC/page-index documents and return their serialized payload.",
    tool_struct = "WendaoDocsGetTocDocumentsTool"
)]
pub fn wendao_docs_get_toc_documents(
    ctx: &ZhenfaContext,
    _args: WendaoDocsGetTocDocumentsArgs,
) -> Result<String, ZhenfaError> {
    let runtime = resolve_docs_tool_runtime(ctx)?;
    let result = runtime
        .get_toc_documents()
        .map_err(|error| ZhenfaError::execution(error.to_string()))?;
    serialize_payload(&result)
}
