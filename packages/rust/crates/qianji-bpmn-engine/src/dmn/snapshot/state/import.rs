use crate::dmn::snapshot::xml::attribute_value;
use crate::dmn_model_api::{DmnImportSnapshot, DmnSourceFile};
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[derive(Debug)]
pub(super) struct TempImportSnapshot {
    name: Option<String>,
    namespace: Option<String>,
    location_uri: Option<String>,
    import_type: Option<String>,
}

impl From<TempImportSnapshot> for DmnImportSnapshot {
    fn from(value: TempImportSnapshot) -> Self {
        Self {
            name: value.name,
            namespace: value.namespace,
            location_uri: value.location_uri,
            import_type: value.import_type,
        }
    }
}

impl TempImportSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            name: attribute_value(source, reader, event, "name")?,
            namespace: attribute_value(source, reader, event, "namespace")?,
            location_uri: attribute_value(source, reader, event, "locationURI")?,
            import_type: attribute_value(source, reader, event, "importType")?,
        })
    }
}
