//! `zhenfa_router::native::docs::navigation` owns Wendao native docs navigation behavior.

use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError, zhenfa_tool};

use super::shared::{optional_non_empty_argument, require_non_empty_argument, serialize_payload};
use crate::analyzers::{DocsNavigationOptions, DocsNavigationToolArgs};
use crate::zhenfa_router::native::resolve_docs_tool_runtime;

/// Arguments for the `wendao.docs.get_navigation` native tool.
pub type WendaoDocsGetNavigationArgs = DocsNavigationToolArgs;

/// Resolve one docs-facing navigation bundle and return its serialized payload.
///
/// # Errors
///
/// Returns a [`ZhenfaError`] when arguments are invalid, the docs capability
/// service is missing from the native context, or the underlying docs lookup
/// fails.
#[allow(missing_docs)]
#[zhenfa_tool(
    name = "wendao.docs.get_navigation",
    description = "Open one docs-facing navigation bundle and return its serialized payload.",
    tool_struct = "WendaoDocsGetNavigationTool"
)]
pub fn wendao_docs_get_navigation(
    ctx: &ZhenfaContext,
    args: WendaoDocsGetNavigationArgs,
) -> Result<String, ZhenfaError> {
    let DocsNavigationToolArgs {
        page_id,
        node_id,
        family_kind,
        related_limit,
        family_limit,
    } = args;
    let page_id = require_non_empty_argument(&page_id, "page_id")?;
    let runtime = resolve_docs_tool_runtime(ctx)?;
    let defaults = DocsNavigationOptions::default();
    let options = DocsNavigationOptions {
        node_id: optional_non_empty_argument(node_id, "node_id")?,
        family_kind,
        related_limit: related_limit.unwrap_or(defaults.related_limit),
        family_limit: family_limit.unwrap_or(defaults.family_limit),
    };
    let result = runtime
        .get_navigation_with_options(&page_id, options)
        .map_err(|error| ZhenfaError::execution(error.to_string()))?;
    serialize_payload(&result)
}
