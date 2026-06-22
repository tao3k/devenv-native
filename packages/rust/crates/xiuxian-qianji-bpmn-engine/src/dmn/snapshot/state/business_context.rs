use crate::dmn::snapshot::xml::attribute_value;
use crate::dmn_model_api::{
    DmnOrganizationUnitSnapshot, DmnPerformanceIndicatorSnapshot, DmnSourceFile,
};
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[derive(Debug)]
pub(super) struct TempOrganizationUnitSnapshot {
    organization_unit_id: Option<String>,
    name: Option<String>,
}

impl From<TempOrganizationUnitSnapshot> for DmnOrganizationUnitSnapshot {
    fn from(value: TempOrganizationUnitSnapshot) -> Self {
        Self {
            organization_unit_id: value.organization_unit_id,
            name: value.name,
        }
    }
}

impl TempOrganizationUnitSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            organization_unit_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
        })
    }
}

#[derive(Debug)]
pub(super) struct TempPerformanceIndicatorSnapshot {
    performance_indicator_id: Option<String>,
    name: Option<String>,
}

impl From<TempPerformanceIndicatorSnapshot> for DmnPerformanceIndicatorSnapshot {
    fn from(value: TempPerformanceIndicatorSnapshot) -> Self {
        Self {
            performance_indicator_id: value.performance_indicator_id,
            name: value.name,
        }
    }
}

impl TempPerformanceIndicatorSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            performance_indicator_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
        })
    }
}
