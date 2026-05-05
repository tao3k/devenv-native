#[path = "model.rs"]
mod model;
#[path = "xml.rs"]
mod xml;
pub(crate) use model::{
    SqlAuthorSpec, SqlFilter, SqlOrderTerm, SurfaceBundle, SurfaceColumn, SurfaceObject,
    SurfacePolicy,
};
#[path = "facade.rs"]
mod facade;
#[cfg(test)]
#[path = "../../tests/unit/executors/wendao_sql/contract.rs"]
mod tests;

pub(crate) use facade::{parse_sql_author_spec_xml, parse_surface_bundle_xml};
