//! `zhenfa_router` owns Wendao zhenfa router behavior.

#[cfg(feature = "zhenfa-router")]
#[path = "http.rs"]
mod http;
#[path = "models.rs"]
mod models;
/// Native Zhenfa router implementations for Wendao.
///
/// This module contains the core logic for semantic operations,
/// search tools, and context extensions specific to the Wendao knowledge graph.
#[path = "native/mod.rs"]
pub mod native;
#[path = "rpc.rs"]
mod rpc;

#[cfg(feature = "zhenfa-router")]
pub use http::WendaoZhenfaRouter;
pub use native::{
    WendaoAgenticNavArgs, WendaoContextExt, WendaoDocsGetDocumentArgs, WendaoDocsGetNavigationArgs,
    WendaoDocsGetPageIndexTreeArgs, WendaoDocsGetRetrievalContextArgs, WendaoPluginArtifactArgs,
    WendaoPluginArtifactOutputFormat, WendaoSearchArgs, audit_search_payload, evaluate_alignment,
    export_plugin_artifact, render_plugin_artifact, render_plugin_artifact_json,
    render_plugin_artifact_toml, render_xml_lite_hits, wendao_agentic_nav,
    wendao_docs_get_document, wendao_docs_get_navigation, wendao_docs_get_page_index_tree,
    wendao_docs_get_retrieval_context, wendao_search,
};
pub use rpc::{execute_search, export_plugin_artifact_from_rpc_params, search_from_rpc_params};
