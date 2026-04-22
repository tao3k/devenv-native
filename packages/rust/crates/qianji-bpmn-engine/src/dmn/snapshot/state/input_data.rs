use crate::dmn::snapshot::xml::attribute_value;
use crate::dmn_model_api::{DmnInputDataSnapshot, DmnSourceFile, DmnVariableSnapshot};
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[derive(Debug)]
pub(super) struct TempInputDataSnapshot {
    input_data_id: Option<String>,
    name: Option<String>,
    variable: Option<DmnVariableSnapshot>,
}

impl From<TempInputDataSnapshot> for DmnInputDataSnapshot {
    fn from(value: TempInputDataSnapshot) -> Self {
        Self {
            input_data_id: value.input_data_id,
            name: value.name,
            variable: value.variable,
        }
    }
}

impl TempInputDataSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            input_data_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            variable: None,
        })
    }

    pub(super) fn set_direct_variable(&mut self, variable: DmnVariableSnapshot) {
        self.variable = Some(variable);
    }
}

pub(super) fn variable_from_event(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<DmnVariableSnapshot> {
    Ok(DmnVariableSnapshot {
        variable_id: attribute_value(source, reader, event, "id")?,
        name: attribute_value(source, reader, event, "name")?,
        type_ref: attribute_value(source, reader, event, "typeRef")?,
    })
}
