//! Julia parser-summary contracts, transport, and incremental helpers.

mod contract;
mod fetch;
mod incremental;
mod route;
mod transport;
mod types;

pub(crate) use fetch::{
    fetch_julia_parser_file_summary_blocking_for_repository,
    fetch_julia_parser_root_summary_blocking_for_repository,
    validate_julia_parser_summary_preflight_for_repository,
};
pub use incremental::{
    julia_parser_summary_allows_safe_incremental_file_for_repository,
    julia_parser_summary_file_semantic_fingerprint_for_repository,
};
pub use transport::set_linked_julia_parser_summary_base_url_for_tests;
pub use types::JuliaSourceId;
pub(crate) use types::{
    JuliaParserDocAttachment, JuliaParserDocTargetKind, JuliaParserFileSummary, JuliaParserImport,
    JuliaParserSourceSummary, JuliaParserSymbol, JuliaParserSymbolKind,
};
