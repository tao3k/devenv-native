//! Studio API endpoint handlers.

mod document_extract;
mod flight;
mod service;

pub(crate) use document_extract::{
    DocumentExtractJobStatus, DocumentExtractRuntimeSnapshot,
    StudioDocumentExtractFlightRouteProvider, default_output_dir,
};
pub(crate) use flight::{
    StudioCodeAstAnalysisFlightRouteProvider, StudioMarkdownAnalysisFlightRouteProvider,
};
pub(crate) use service::{load_code_ast_analysis_response, load_markdown_analysis_response};
