//! Studio API endpoint handlers.

#[path = "document_extract/mod.rs"]
mod document_extract;
mod flight;
#[path = "service/mod.rs"]
mod service;

pub(crate) use document_extract::{
    DocumentExtractJobStatus, DocumentExtractRuntimeSnapshot,
    StudioDocumentExtractFlightRouteProvider, default_output_dir,
};
pub(crate) use flight::{
    StudioCodeAstAnalysisFlightRouteProvider, StudioMarkdownAnalysisFlightRouteProvider,
};
pub(crate) use service::{load_code_ast_analysis_response, load_markdown_analysis_response};
