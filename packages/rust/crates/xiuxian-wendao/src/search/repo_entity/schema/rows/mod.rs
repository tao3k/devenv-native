//! `search::repo_entity::schema::rows` owns Wendao repo entity schema rows behavior.

mod context;
mod definition;
mod example;
mod import;
mod module;
mod orchestration;
mod symbol;

pub(crate) use context::RepoEntityContext;
pub(crate) use definition::repo_entity_schema;
pub(crate) use example::build_example_row;
pub(crate) use import::build_import_row;
pub(crate) use module::build_module_row;
pub(crate) use orchestration::rows_from_analysis;
pub(crate) use symbol::build_symbol_row;
