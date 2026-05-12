//! `zhenfa_router::native::docs::segment` owns Wendao native docs segment behavior.

use schemars::JsonSchema;
use serde::Deserialize;
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError, zhenfa_tool};

use super::shared::{require_non_empty_argument, serialize_payload};
use crate::zhenfa_router::native::resolve_docs_tool_runtime;

/// Arguments for the `wendao.docs.get_document_segment` native tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WendaoDocsGetDocumentSegmentArgs {
    /// Stable docs-facing page identifier.
    page_id: String,
    /// 1-based inclusive line-range start.
    line_start: usize,
    /// 1-based inclusive line-range end.
    line_end: usize,
}

/// Resolve one precise docs-facing projected markdown segment and return its
/// serialized payload.
///
/// # Errors
///
/// Returns a [`ZhenfaError`] when arguments are invalid, the docs capability
/// service is missing from the native context, or the underlying docs lookup
/// fails.
#[allow(missing_docs)]
#[zhenfa_tool(
    name = "wendao.docs.get_document_segment",
    description = "Open one precise docs-facing projected markdown segment and return its serialized payload.",
    tool_struct = "WendaoDocsGetDocumentSegmentTool"
)]
pub fn wendao_docs_get_document_segment(
    ctx: &ZhenfaContext,
    args: WendaoDocsGetDocumentSegmentArgs,
) -> Result<String, ZhenfaError> {
    let WendaoDocsGetDocumentSegmentArgs {
        page_id,
        line_start,
        line_end,
    } = args;
    let page_id = require_non_empty_argument(&page_id, "page_id")?;
    let runtime = resolve_docs_tool_runtime(ctx)?;
    let result = runtime
        .get_document_segment(&page_id, line_start, line_end)
        .map_err(|error| ZhenfaError::execution(error.to_string()))?;
    serialize_payload(&result)
}
