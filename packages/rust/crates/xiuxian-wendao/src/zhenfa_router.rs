#[cfg(feature = "zhenfa-router")]
#[path = "zhenfa_router/http.rs"]
mod http;
#[path = "zhenfa_router/models.rs"]
mod models;
/// Native Zhenfa router implementations for Wendao.
///
/// This module contains the core logic for semantic operations,
/// search tools, and context extensions specific to the Wendao knowledge graph.
#[path = "zhenfa_router/native.rs"]
pub mod native;
#[path = "zhenfa_router/rpc.rs"]
mod rpc;

#[cfg(feature = "zhenfa-router")]
pub use http::WendaoZhenfaRouter;
pub use native::{
    WendaoAgenticNavTool, WendaoContextExt, WendaoDocsGetDocumentArgs,
    WendaoDocsGetDocumentStructureArgs, WendaoDocsGetDocumentStructureTool,
    WendaoDocsGetDocumentTool, WendaoDocsGetNavigationArgs, WendaoDocsGetNavigationTool,
    WendaoDocsGetRetrievalContextArgs, WendaoDocsGetRetrievalContextTool, WendaoPluginArtifactArgs,
    WendaoPluginArtifactOutputFormat, WendaoPluginArtifactTool, WendaoSearchTool,
    WendaoSemanticCheckTool, WendaoSemanticEditTool, WendaoSemanticReadTool, audit_search_payload,
    evaluate_alignment, export_plugin_artifact, register_wendao_docs_native_tools,
    render_plugin_artifact, render_plugin_artifact_json, render_plugin_artifact_toml,
    render_xml_lite_hits, wendao_docs_get_document, wendao_docs_get_document_structure,
    wendao_docs_get_navigation, wendao_docs_get_retrieval_context,
};
pub use rpc::{execute_search, export_plugin_artifact_from_rpc_params, search_from_rpc_params};
