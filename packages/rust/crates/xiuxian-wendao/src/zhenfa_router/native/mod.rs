//! Native Zhenfa router tools for Wendao.
//!
//! This module keeps the public tool surface stable while the implementation
//! is organized into feature-focused leaf modules.

#[path = "agentic_nav.rs"]
mod agentic_nav;
/// Public Wendao boundary.
#[path = "audit/mod.rs"]
pub mod audit;
#[path = "context.rs"]
mod context;
#[path = "deployment.rs"]
mod deployment;
#[path = "docs/mod.rs"]
mod docs;
#[path = "forwarder/mod.rs"]
mod forwarder;
#[path = "remediation.rs"]
mod remediation;
#[path = "search.rs"]
mod search;
/// Public Wendao boundary.
#[path = "semantic_check/mod.rs"]
pub mod semantic_check;
#[path = "semantic_edit.rs"]
mod semantic_edit;
#[path = "semantic_read.rs"]
mod semantic_read;
/// Public Wendao boundary.
#[path = "sentinel/mod.rs"]
pub mod sentinel;
#[path = "xml_lite.rs"]
mod xml_lite;

pub use agentic_nav::{WendaoAgenticNavArgs, wendao_agentic_nav};
pub use audit::{audit_search_payload, evaluate_alignment};
pub use context::WendaoContextExt;
pub(crate) use context::resolve_docs_tool_runtime;
pub use deployment::{
    WendaoPluginArtifactArgs, WendaoPluginArtifactOutputFormat, export_plugin_artifact,
    render_plugin_artifact, render_plugin_artifact_json, render_plugin_artifact_toml,
    wendao_plugin_artifact,
};
pub use docs::{
    WendaoDocsGetDocumentArgs, WendaoDocsGetDocumentNodeArgs, WendaoDocsGetDocumentSegmentArgs,
    WendaoDocsGetNavigationArgs, WendaoDocsGetPageIndexArgs, WendaoDocsGetPageIndexOutlineArgs,
    WendaoDocsGetPageIndexTreeArgs, WendaoDocsGetRetrievalContextArgs,
    WendaoDocsGetTocDocumentsArgs, WendaoDocsSearchArgs, WendaoDocsSearchPageIndexArgs,
    wendao_docs_get_document, wendao_docs_get_document_node, wendao_docs_get_document_segment,
    wendao_docs_get_navigation, wendao_docs_get_page_index, wendao_docs_get_page_index_outline,
    wendao_docs_get_page_index_tree, wendao_docs_get_retrieval_context,
    wendao_docs_get_toc_documents, wendao_docs_search, wendao_docs_search_page_index,
};
pub use forwarder::{
    AffectedDocInfo, ForwardNotification, ForwardNotifier, ForwarderConfig, SuggestedAction,
};
pub use remediation::{
    RemediationAction, RemediationConfig, RemediationContextExt, RemediationResult,
    RemediationWorker,
};
pub use search::{WendaoSearchArgs, render_xml_lite_hits, wendao_search};
pub use semantic_edit::{WendaoSemanticEditArgs, wendao_semantic_edit};
pub use semantic_read::{WendaoSemanticReadArgs, wendao_semantic_read};
pub use sentinel::{
    AffectedDoc, DriftConfidence, ObservationBus, ObservationRef, ObservationSignal,
    SemanticDriftSignal, propagate_source_change, signals_to_status_batch,
};
