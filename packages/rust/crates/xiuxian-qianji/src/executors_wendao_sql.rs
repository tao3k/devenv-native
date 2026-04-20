#[path = "executors/wendao_sql/contract.rs"]
mod contract;
#[path = "executors/wendao_sql/discover.rs"]
mod discover;
#[path = "executors/wendao_sql/execute.rs"]
mod execute;
#[path = "executors/wendao_sql/gateway.rs"]
mod gateway;
#[path = "executors/wendao_sql/input.rs"]
mod input;
#[path = "executors/wendao_sql/render.rs"]
mod render;
#[path = "executors/wendao_sql/validate.rs"]
mod validate;

#[cfg(test)]
pub(crate) use contract::{parse_sql_author_spec_xml, parse_surface_bundle_xml};
pub use discover::WendaoSqlDiscoverMechanism;
pub use execute::WendaoSqlExecuteMechanism;
pub use validate::WendaoSqlValidateMechanism;
