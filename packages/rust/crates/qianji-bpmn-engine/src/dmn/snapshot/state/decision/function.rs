use super::{
    BytesStart, DmnFunctionDefinitionLiteralSnapshot, DmnFunctionDefinitionParameterSnapshot,
    DmnFunctionDefinitionSnapshot, DmnSourceFile, Reader, Result, attribute_value, non_empty_text,
};

#[derive(Debug)]
pub(in crate::dmn::snapshot::state) struct TempFunctionDefinitionSnapshot {
    function_definition_id: Option<String>,
    kind: Option<String>,
    parameters: Vec<TempFunctionDefinitionParameterSnapshot>,
    body: Option<TempFunctionDefinitionLiteralSnapshot>,
}

impl From<TempFunctionDefinitionSnapshot> for DmnFunctionDefinitionSnapshot {
    fn from(value: TempFunctionDefinitionSnapshot) -> Self {
        Self {
            function_definition_id: value.function_definition_id,
            kind: (value.kind.map(Into::into)),
            parameters: value.parameters.into_iter().map(Into::into).collect(),
            body: value.body.map(Into::into),
        }
    }
}

impl TempFunctionDefinitionSnapshot {
    pub(in crate::dmn::snapshot::state) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            function_definition_id: attribute_value(source, reader, event, "id")?,
            kind: attribute_value(source, reader, event, "kind")?,
            parameters: Vec::new(),
            body: None,
        })
    }

    pub(in crate::dmn::snapshot::state) fn push_parameter(
        &mut self,
        parameter: TempFunctionDefinitionParameterSnapshot,
    ) {
        self.parameters.push(parameter);
    }

    pub(in crate::dmn::snapshot::state) fn set_body(
        &mut self,
        body: TempFunctionDefinitionLiteralSnapshot,
    ) {
        self.body = Some(body);
    }
}

#[derive(Debug)]
pub(in crate::dmn::snapshot::state) struct TempFunctionDefinitionParameterSnapshot {
    parameter_id: Option<String>,
    name: Option<String>,
    type_ref: Option<String>,
}

impl From<TempFunctionDefinitionParameterSnapshot> for DmnFunctionDefinitionParameterSnapshot {
    fn from(value: TempFunctionDefinitionParameterSnapshot) -> Self {
        Self {
            parameter_id: value.parameter_id,
            name: value.name,
            type_ref: value.type_ref,
        }
    }
}

impl TempFunctionDefinitionParameterSnapshot {
    pub(in crate::dmn::snapshot::state) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            parameter_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            type_ref: attribute_value(source, reader, event, "typeRef")?,
        })
    }
}

#[derive(Debug)]
pub(in crate::dmn::snapshot::state) struct TempFunctionDefinitionLiteralSnapshot {
    expression_id: Option<String>,
    type_ref: Option<String>,
    text: String,
}

impl From<TempFunctionDefinitionLiteralSnapshot> for DmnFunctionDefinitionLiteralSnapshot {
    fn from(value: TempFunctionDefinitionLiteralSnapshot) -> Self {
        let text = non_empty_text(&value.text);
        Self {
            expression_id: value.expression_id,
            type_ref: value.type_ref,
            text,
        }
    }
}

impl TempFunctionDefinitionLiteralSnapshot {
    pub(in crate::dmn::snapshot::state) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            expression_id: attribute_value(source, reader, event, "id")?,
            type_ref: attribute_value(source, reader, event, "typeRef")?,
            text: String::new(),
        })
    }

    pub(in crate::dmn::snapshot::state) fn append_text(&mut self, text: &str) {
        self.text.push_str(text);
    }
}
