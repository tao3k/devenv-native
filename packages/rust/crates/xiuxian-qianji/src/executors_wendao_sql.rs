//! Wendao SQL executor branch that owns contract, validation, and execution flow.

#[path = "executors_wendao_sql_contract/mod.rs"]
mod contract;
#[path = "executors_wendao_sql_discover.rs"]
mod discover;
#[path = "executors_wendao_sql_execute.rs"]
mod execute;
#[path = "executors/wendao_sql/gateway.rs"]
mod gateway;
#[path = "executors/wendao_sql/input.rs"]
mod input;
#[path = "executors/wendao_sql/render.rs"]
mod render;
#[path = "executors_wendao_sql_validate/mod.rs"]
mod validate;

#[cfg(test)]
pub(crate) use contract::{parse_sql_author_spec_xml, parse_surface_bundle_xml};
pub use discover::WendaoSqlDiscoverMechanism;
pub use execute::WendaoSqlExecuteMechanism;
pub use validate::WendaoSqlValidateMechanism;
