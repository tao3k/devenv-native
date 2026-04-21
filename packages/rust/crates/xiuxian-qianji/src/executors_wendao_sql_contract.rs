#[path = "executors_wendao_sql_contract/model.rs"]
mod model;
#[path = "executors_wendao_sql_contract/xml.rs"]
mod xml;

use quick_xml::de::from_str;

pub(crate) use model::{
    SqlAuthorSpec, SqlFilter, SqlOrderTerm, SurfaceBundle, SurfaceColumn, SurfaceObject,
    SurfacePolicy,
};
use xml::{SqlAuthorSpecXml, SurfaceBundleXml};

pub(crate) fn parse_surface_bundle_xml(raw: &str) -> Result<SurfaceBundle, String> {
    from_str::<SurfaceBundleXml>(raw)
        .map(Into::into)
        .map_err(|error| format!("failed to parse surface bundle XML: {error}"))
}

pub(crate) fn parse_sql_author_spec_xml(raw: &str) -> Result<SqlAuthorSpec, String> {
    from_str::<SqlAuthorSpecXml>(raw)
        .map(Into::into)
        .map_err(|error| format!("failed to parse sql author spec XML: {error}"))
}

#[cfg(test)]
#[path = "../tests/unit/executors/wendao_sql/contract.rs"]
mod tests;
