//! `search::repo_entity::query::results` owns Wendao repo entity query results behavior.

mod example;
mod import;
mod module;
mod overview;
mod shared;
mod symbol;

pub(crate) use example::search_repo_entity_example_results;
pub(crate) use import::search_repo_entity_import_results;
pub(crate) use module::search_repo_entity_module_results;
pub use overview::{RepoEntityOverviewSummary, summarize_repo_entity_overview};
pub(crate) use symbol::search_repo_entity_symbol_results;
