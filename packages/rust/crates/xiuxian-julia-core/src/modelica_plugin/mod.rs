//! Modelica repository intelligence plugin owner tree.

mod analysis;
mod discovery;
mod entry;
mod incremental;
mod parser_summary;
mod parsing;
mod pathing;
mod relations;
mod types;

pub use entry::{ModelicaRepoIntelligencePlugin, register_modelica_into};
pub use incremental::{
    modelica_package_incremental_semantic_fingerprint_for_repository,
    modelica_parser_summary_allows_safe_incremental_file_for_repository,
    modelica_parser_summary_allows_safe_package_incremental_file_for_repository,
    modelica_parser_summary_allows_safe_root_package_incremental_file_for_repository,
    modelica_parser_summary_root_package_name_matches_repository_context,
    modelica_root_package_incremental_semantic_fingerprint_for_repository,
};
pub(crate) use parser_summary::fetch_modelica_parser_file_summary_blocking_for_repository;
#[cfg(test)]
pub(crate) use parser_summary::{
    MODELICA_FILE_SUMMARY_ROUTE, MODELICA_PARSER_SUMMARY_SCHEMA_VERSION,
};
pub use parser_summary::{
    clear_modelica_parser_summary_transport_cache_for_tests,
    modelica_parser_summary_file_semantic_fingerprint_for_repository,
    set_linked_modelica_parser_summary_base_url_for_tests,
};
pub use types::ModelicaSourceId;
