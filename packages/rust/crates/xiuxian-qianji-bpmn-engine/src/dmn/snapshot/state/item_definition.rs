use crate::dmn::snapshot::xml::attribute_value;
use crate::dmn_model_api::{DmnItemComponentSnapshot, DmnItemDefinitionSnapshot, DmnSourceFile};
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[derive(Debug)]
pub(super) struct TempItemDefinitionSnapshot {
    item_definition_id: Option<String>,
    name: Option<String>,
    type_ref: Option<String>,
    is_collection: Option<bool>,
    item_components: Vec<DmnItemComponentSnapshot>,
}

impl From<TempItemDefinitionSnapshot> for DmnItemDefinitionSnapshot {
    fn from(value: TempItemDefinitionSnapshot) -> Self {
        Self {
            item_definition_id: value.item_definition_id,
            name: value.name,
            type_ref: value.type_ref,
            is_collection: value.is_collection,
            item_components: value.item_components,
        }
    }
}

impl TempItemDefinitionSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            item_definition_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            type_ref: attribute_value(source, reader, event, "typeRef")?,
            is_collection: optional_bool_attribute(source, reader, event, "isCollection")?,
            item_components: Vec::new(),
        })
    }

    pub(super) fn push_direct_item_component(&mut self, item_component: DmnItemComponentSnapshot) {
        self.item_components.push(item_component);
    }
}

fn parse_optional_bool(value: Option<&str>) -> Option<bool> {
    match value {
        Some("true") => Some(true),
        Some("false") => Some(false),
        _ => None,
    }
}

fn optional_bool_attribute(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    attribute_name: &str,
) -> Result<Option<bool>> {
    let raw = attribute_value(source, reader, event, attribute_name)?;
    Ok(parse_optional_bool(raw.as_deref()))
}

pub(super) fn item_component_from_event(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<DmnItemComponentSnapshot> {
    Ok(DmnItemComponentSnapshot {
        item_component_id: attribute_value(source, reader, event, "id")?,
        name: attribute_value(source, reader, event, "name")?,
        type_ref: attribute_value(source, reader, event, "typeRef")?,
    })
}
