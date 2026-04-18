//! Search backend integration for Studio API.

#[path = "handlers/ast/mod.rs"]
mod ast;
#[path = "handlers/attachments/mod.rs"]
mod attachments;
#[path = "handlers/autocomplete/mod.rs"]
mod autocomplete;
#[path = "handlers/code_search/mod.rs"]
mod code_search;
#[path = "handlers/definition/mod.rs"]
mod definition;
#[path = "handlers/flight/mod.rs"]
mod flight;
#[path = "handlers/index.rs"]
mod index;
#[path = "handlers/knowledge/mod.rs"]
mod knowledge;
#[path = "handlers/queries/mod.rs"]
mod queries;
#[path = "handlers/references/mod.rs"]
mod references;
#[path = "handlers/status.rs"]
mod status;
#[path = "handlers/symbols/mod.rs"]
mod symbols;
#[cfg(test)]
#[path = "../../../../tests/unit/gateway/studio/search/handlers/test_prelude.rs"]
mod test_prelude;

#[cfg(test)]
pub use ast::search_ast;
#[cfg(test)]
pub(crate) use attachments::load_attachment_search_response_from_studio;
#[cfg(test)]
pub(crate) use autocomplete::build_autocomplete_response;
#[cfg(test)]
pub(crate) use definition::build_definition_response;
pub use flight::{
    StudioRepoSearchFlightRouteProvider, bootstrap_sample_repo_search_content,
    build_repo_search_flight_service, build_repo_search_flight_service_with_weights,
    build_studio_flight_service, build_studio_flight_service_for_roots,
    build_studio_flight_service_for_roots_with_weights, build_studio_flight_service_with_weights,
};
#[cfg(test)]
pub use index::build_ast_index;
pub use index::build_symbol_index;
#[cfg(test)]
pub(crate) use knowledge::build_knowledge_search_response;
#[cfg(test)]
pub(crate) use knowledge::load_intent_search_response_with_metadata;
#[cfg(test)]
pub(crate) use references::load_reference_search_response;
pub use status::search_index_status;
#[cfg(test)]
pub(crate) use symbols::load_symbol_search_response;

#[cfg(test)]
#[path = "../../../../tests/unit/gateway/studio/search/mod.rs"]
mod studio_search_tests;

#[cfg(test)]
#[path = "../../../../tests/unit/gateway/studio/search/handlers/mod.rs"]
pub(crate) mod tests;
