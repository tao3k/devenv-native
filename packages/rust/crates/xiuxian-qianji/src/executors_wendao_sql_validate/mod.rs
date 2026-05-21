//! Wendao SQL validation branch for render, gate, and facade logic.

#[path = "render_sql.rs"]
mod render_sql;
#[path = "validation.rs"]
mod validation;
use super::contract::{SqlAuthorSpec, SqlFilter, SqlOrderTerm, SurfaceBundle, SurfaceColumn};
#[path = "facade.rs"]
mod facade;

pub use facade::WendaoSqlValidateMechanism;
