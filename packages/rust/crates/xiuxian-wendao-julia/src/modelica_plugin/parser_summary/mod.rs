mod contract;
mod fetch;
mod incremental;
mod transport;
mod types;

pub(crate) use fetch::{
    fetch_modelica_parser_file_summary_blocking_for_repository,
    validate_modelica_parser_summary_preflight_for_repository,
};
pub use incremental::modelica_parser_summary_file_semantic_fingerprint_for_repository;
#[cfg(test)]
pub(crate) use transport::{
    MODELICA_AST_QUERY_ROUTE, MODELICA_FILE_SUMMARY_ROUTE, MODELICA_PARSER_SUMMARY_SCHEMA_VERSION,
};
pub(crate) use transport::{
    ParserSummaryRouteKind, modelica_parser_summary_timeout_secs_for_repository,
    process_modelica_parser_summary_flight_batches_for_repository,
};
pub use transport::{
    clear_modelica_parser_summary_transport_cache_for_tests,
    set_linked_modelica_parser_summary_base_url_for_tests,
};
