//! Studio API endpoint handlers.

pub(crate) mod document_extract;
mod flight;
mod service;

pub(crate) use document_extract::StudioDocumentExtractFlightRouteProvider;
pub(crate) use flight::{
    StudioCodeAstAnalysisFlightRouteProvider, StudioMarkdownAnalysisFlightRouteProvider,
};
pub(crate) use service::{load_code_ast_analysis_response, load_markdown_analysis_response};
