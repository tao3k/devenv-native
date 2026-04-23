#[path = "engine/compiler/wendao_sql/discover.rs"]
mod discover;
#[path = "engine/compiler/wendao_sql/execute.rs"]
mod execute;
#[path = "engine/compiler/wendao_sql/shared.rs"]
mod shared;
#[path = "engine/compiler/wendao_sql/validate.rs"]
mod validate;

pub(in crate::engine::compiler) use discover::mechanism_config as discover_mechanism_config;
pub(in crate::engine::compiler) use execute::mechanism_config as execute_mechanism_config;
pub(in crate::engine::compiler) use validate::mechanism_config as validate_mechanism_config;
