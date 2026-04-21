//! Studio API endpoint handlers.

mod flight;
pub(crate) mod pdf_extract;
mod service;

pub(crate) use flight::{
    StudioCodeAstAnalysisFlightRouteProvider, StudioMarkdownAnalysisFlightRouteProvider,
};
pub(crate) use pdf_extract::StudioPdfExtractFlightRouteProvider;
pub(crate) use service::{load_code_ast_analysis_response, load_markdown_analysis_response};
