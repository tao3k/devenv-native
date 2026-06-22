use crate::dmn::snapshot::xml::attribute_value;
use crate::dmn_model_api::{
    DmnAssociationSnapshot, DmnElementCollectionSnapshot, DmnGroupSnapshot, DmnSourceFile,
};
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[derive(Debug)]
pub(super) struct TempAssociationSnapshot {
    association_id: Option<String>,
    association_direction: Option<String>,
    source_ref: String,
    target_ref: String,
}

impl From<TempAssociationSnapshot> for DmnAssociationSnapshot {
    fn from(value: TempAssociationSnapshot) -> Self {
        Self {
            association_id: value.association_id,
            association_direction: value.association_direction,
            source_ref: (!value.source_ref.is_empty()).then_some(value.source_ref),
            target_ref: (!value.target_ref.is_empty()).then_some(value.target_ref),
        }
    }
}

impl TempAssociationSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            association_id: attribute_value(source, reader, event, "id")?,
            association_direction: attribute_value(source, reader, event, "associationDirection")?,
            source_ref: String::new(),
            target_ref: String::new(),
        })
    }

    pub(super) fn append_source_ref(&mut self, text: &str) {
        self.source_ref.push_str(text);
    }

    pub(super) fn append_target_ref(&mut self, text: &str) {
        self.target_ref.push_str(text);
    }
}

#[derive(Debug)]
pub(super) struct TempElementCollectionSnapshot {
    element_collection_id: Option<String>,
    name: Option<String>,
}

impl From<TempElementCollectionSnapshot> for DmnElementCollectionSnapshot {
    fn from(value: TempElementCollectionSnapshot) -> Self {
        Self {
            element_collection_id: value.element_collection_id,
            name: value.name,
        }
    }
}

impl TempElementCollectionSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            element_collection_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
        })
    }
}

#[derive(Debug)]
pub(super) struct TempGroupSnapshot {
    group_id: Option<String>,
    name: Option<String>,
}

impl From<TempGroupSnapshot> for DmnGroupSnapshot {
    fn from(value: TempGroupSnapshot) -> Self {
        Self {
            group_id: value.group_id,
            name: value.name,
        }
    }
}

impl TempGroupSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            group_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
        })
    }
}
