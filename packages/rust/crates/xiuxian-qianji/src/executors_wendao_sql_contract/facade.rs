use super::xml::{SqlAuthorSpecXml, SurfaceBundleXml};

use quick_xml::de::from_str;

use super::{SqlAuthorSpec, SurfaceBundle};

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
