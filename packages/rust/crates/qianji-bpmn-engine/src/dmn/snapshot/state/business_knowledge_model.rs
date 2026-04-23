use crate::dmn::snapshot::xml::attribute_value;
use crate::dmn_model_api::{DmnBusinessKnowledgeModelSnapshot, DmnSourceFile};
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[derive(Debug)]
pub(super) struct TempBusinessKnowledgeModelSnapshot {
    business_knowledge_model_id: Option<String>,
    name: Option<String>,
}

impl From<TempBusinessKnowledgeModelSnapshot> for DmnBusinessKnowledgeModelSnapshot {
    fn from(value: TempBusinessKnowledgeModelSnapshot) -> Self {
        Self {
            business_knowledge_model_id: value.business_knowledge_model_id,
            name: value.name,
        }
    }
}

impl TempBusinessKnowledgeModelSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            business_knowledge_model_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
        })
    }
}
