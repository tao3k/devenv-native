use crate::dmn::snapshot::xml::attribute_value;
use crate::dmn_model_api::{DmnKnowledgeSourceSnapshot, DmnSourceFile};
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[derive(Debug)]
pub(super) struct TempKnowledgeSourceSnapshot {
    knowledge_source_id: Option<String>,
    name: Option<String>,
}

impl From<TempKnowledgeSourceSnapshot> for DmnKnowledgeSourceSnapshot {
    fn from(value: TempKnowledgeSourceSnapshot) -> Self {
        Self {
            knowledge_source_id: value.knowledge_source_id,
            name: value.name,
        }
    }
}

impl TempKnowledgeSourceSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            knowledge_source_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
        })
    }
}
