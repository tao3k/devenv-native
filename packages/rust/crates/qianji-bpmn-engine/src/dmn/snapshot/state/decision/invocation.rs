use super::{
    BytesStart, DmnInvocationBindingSnapshot, DmnInvocationLiteralSnapshot,
    DmnInvocationParameterSnapshot, DmnInvocationSnapshot, DmnSourceFile, Reader, Result,
    attribute_value, non_empty_text,
};

pub(in crate::dmn::snapshot::state) struct TempInvocationSnapshot {
    invocation_id: Option<String>,
    invoked_expression: Option<TempInvocationLiteralSnapshot>,
    bindings: Vec<TempInvocationBindingSnapshot>,
}

impl From<TempInvocationSnapshot> for DmnInvocationSnapshot {
    fn from(value: TempInvocationSnapshot) -> Self {
        Self {
            invocation_id: value.invocation_id,
            invoked_expression: value.invoked_expression.map(Into::into),
            bindings: value.bindings.into_iter().map(Into::into).collect(),
        }
    }
}

impl TempInvocationSnapshot {
    pub(in crate::dmn::snapshot::state) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            invocation_id: attribute_value(source, reader, event, "id")?,
            invoked_expression: None,
            bindings: Vec::new(),
        })
    }

    pub(in crate::dmn::snapshot::state) fn set_invoked_expression(
        &mut self,
        expression: TempInvocationLiteralSnapshot,
    ) {
        self.invoked_expression = Some(expression);
    }

    pub(in crate::dmn::snapshot::state) fn push_binding(
        &mut self,
        binding: TempInvocationBindingSnapshot,
    ) {
        self.bindings.push(binding);
    }
}

#[derive(Debug)]
pub(in crate::dmn::snapshot::state) struct TempInvocationBindingSnapshot {
    binding_id: Option<String>,
    parameter: Option<TempInvocationParameterSnapshot>,
    argument: Option<TempInvocationLiteralSnapshot>,
}

impl From<TempInvocationBindingSnapshot> for DmnInvocationBindingSnapshot {
    fn from(value: TempInvocationBindingSnapshot) -> Self {
        Self {
            binding_id: value.binding_id,
            parameter: value.parameter.map(Into::into),
            argument: value.argument.map(Into::into),
        }
    }
}

impl TempInvocationBindingSnapshot {
    pub(in crate::dmn::snapshot::state) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            binding_id: attribute_value(source, reader, event, "id")?,
            parameter: None,
            argument: None,
        })
    }

    pub(in crate::dmn::snapshot::state) fn set_parameter(
        &mut self,
        parameter: TempInvocationParameterSnapshot,
    ) {
        self.parameter = Some(parameter);
    }

    pub(in crate::dmn::snapshot::state) fn set_argument(
        &mut self,
        argument: TempInvocationLiteralSnapshot,
    ) {
        self.argument = Some(argument);
    }
}

#[derive(Debug)]
pub(in crate::dmn::snapshot::state) struct TempInvocationParameterSnapshot {
    parameter_id: Option<String>,
    name: Option<String>,
    type_ref: Option<String>,
}

impl From<TempInvocationParameterSnapshot> for DmnInvocationParameterSnapshot {
    fn from(value: TempInvocationParameterSnapshot) -> Self {
        Self {
            parameter_id: value.parameter_id,
            name: value.name,
            type_ref: value.type_ref,
        }
    }
}

impl TempInvocationParameterSnapshot {
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
pub(in crate::dmn::snapshot::state) struct TempInvocationLiteralSnapshot {
    expression_id: Option<String>,
    type_ref: Option<String>,
    text: String,
}

impl From<TempInvocationLiteralSnapshot> for DmnInvocationLiteralSnapshot {
    fn from(value: TempInvocationLiteralSnapshot) -> Self {
        let text = non_empty_text(&value.text);
        Self {
            expression_id: value.expression_id,
            type_ref: value.type_ref,
            text,
        }
    }
}

impl TempInvocationLiteralSnapshot {
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
