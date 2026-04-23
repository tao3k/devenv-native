use crate::dmn::snapshot::xml::attribute_value;
use crate::dmn_model_api::{DmnDecisionServiceSnapshot, DmnSourceFile};
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[derive(Debug)]
pub(super) struct TempDecisionServiceSnapshot {
    decision_service_id: Option<String>,
    name: Option<String>,
}

impl From<TempDecisionServiceSnapshot> for DmnDecisionServiceSnapshot {
    fn from(value: TempDecisionServiceSnapshot) -> Self {
        Self {
            decision_service_id: value.decision_service_id,
            name: value.name,
        }
    }
}

impl TempDecisionServiceSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            decision_service_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
        })
    }
}
