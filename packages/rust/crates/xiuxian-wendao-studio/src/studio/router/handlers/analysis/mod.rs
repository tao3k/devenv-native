//! Studio API endpoint handlers.

#[path = "document_extract/mod.rs"]
mod document_extract;
mod flight;
#[path = "service/mod.rs"]
mod service;

#[cfg(test)]
#[path = "../../../../../tests/unit/studio/router/handlers/analysis/flight.rs"]
mod flight_tests;

pub(crate) use document_extract::{
    DocumentExtractJobStatus, DocumentExtractRuntimeSnapshot,
    StudioDocumentExtractFlightRouteProvider, default_output_dir,
};
pub(crate) use flight::{
    StudioMarkdownAnalysisFlightRouteProvider, StudioSemanticScopeFlightRouteProvider,
};
pub(crate) use service::load_markdown_analysis_response;
