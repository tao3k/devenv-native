use super::{
    BoxedExpressionPeerState, BpmnEngineError, BytesStart, DmnSourceFile,
    InvocationChildStartScope, Reader, Result, SurfaceStartState, TempInvocation,
    TempInvocationBinding, TempInvocationParameter, TempLiteralExpression, attribute_value,
};

pub(super) fn handle_invocation_child_start_tag(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    parent_tag: Option<&str>,
    scope: InvocationChildStartScope<'_>,
) -> Result<Option<bool>> {
    let InvocationChildStartScope {
        literal,
        invocation,
        binding,
    } = scope;
    match parent_tag {
        Some("invocation") => match tag {
            "binding" => {
                start_invocation_binding(source, reader, event, invocation.as_ref(), binding)
                    .map(Some)
            }
            "literalExpression" => start_invocation_literal_expression(
                source,
                reader,
                event,
                literal,
                invocation.as_ref(),
            )
            .map(Some),
            _ if invocation.is_some() => Err(BpmnEngineError::UnsupportedOperation {
                operation: "parse_dmn_invocation_unsupported_child",
            }),
            _ => Ok(Some(false)),
        },
        Some("binding") => match tag {
            "parameter" => start_invocation_parameter(source, reader, event, binding).map(Some),
            "literalExpression" => {
                start_invocation_binding_argument(source, reader, event, literal, binding.as_ref())
                    .map(Some)
            }
            _ if binding.is_some() => Err(BpmnEngineError::UnsupportedOperation {
                operation: "parse_dmn_invocation_binding_unsupported_child",
            }),
            _ => Ok(Some(false)),
        },
        _ => Ok(None),
    }
}

pub(super) fn start_invocation_expression(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    surface: SurfaceStartState<'_>,
    current_invocation: &mut Option<TempInvocation>,
    current_invocation_binding: Option<&TempInvocationBinding>,
    peers: BoxedExpressionPeerState<'_>,
) -> Result<bool> {
    let Some(decision) = surface.decision else {
        return Ok(true);
    };
    if decision.table.is_some()
        || decision.literal_expression.is_some()
        || decision.list_expression.is_some()
        || decision.context_expression.is_some()
        || decision.relation_expression.is_some()
        || decision.invocation.is_some()
        || surface.table.is_some()
        || surface.literal.is_some()
        || surface.invocation.is_some()
        || current_invocation.is_some()
        || current_invocation_binding.is_some()
        || peers.list.is_some()
        || peers.context.is_some()
        || peers.relation.is_some()
    {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_invocation_mixed_with_other_executable_surface",
        });
    }
    *current_invocation = Some(TempInvocation {
        invocation_id: attribute_value(source, reader, event, "id")?,
        invoked_expression: None,
        bindings: Vec::new(),
    });
    Ok(true)
}

pub(super) fn start_invocation_binding(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    current_invocation: Option<&TempInvocation>,
    current_invocation_binding: &mut Option<TempInvocationBinding>,
) -> Result<bool> {
    if current_invocation.is_none() {
        return Ok(true);
    }
    if current_invocation_binding.is_some() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_invocation_nested_binding",
        });
    }
    *current_invocation_binding = Some(TempInvocationBinding {
        binding_id: attribute_value(source, reader, event, "id")?,
        parameter: None,
        argument: None,
    });
    Ok(true)
}

pub(super) fn start_invocation_parameter(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    current_invocation_binding: &mut Option<TempInvocationBinding>,
) -> Result<bool> {
    let Some(binding) = current_invocation_binding.as_mut() else {
        return Ok(true);
    };
    binding.parameter = Some(TempInvocationParameter {
        parameter_id: attribute_value(source, reader, event, "id")?,
        name: attribute_value(source, reader, event, "name")?,
        type_ref: attribute_value(source, reader, event, "typeRef")?,
    });
    Ok(true)
}

pub(super) fn start_invocation_literal_expression(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    current_literal: &mut Option<TempLiteralExpression>,
    current_invocation: Option<&TempInvocation>,
) -> Result<bool> {
    if current_invocation.is_none() {
        return Ok(true);
    }
    if current_literal.is_some() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_invocation_nested_literal_expression",
        });
    }
    *current_literal = Some(TempLiteralExpression {
        expression_id: attribute_value(source, reader, event, "id")?,
        type_ref: attribute_value(source, reader, event, "typeRef")?,
        text: None,
    });
    Ok(true)
}

pub(super) fn start_invocation_binding_argument(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    current_literal: &mut Option<TempLiteralExpression>,
    current_invocation_binding: Option<&TempInvocationBinding>,
) -> Result<bool> {
    if current_invocation_binding.is_none() {
        return Ok(true);
    }
    if current_literal.is_some() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_invocation_nested_literal_expression",
        });
    }
    *current_literal = Some(TempLiteralExpression {
        expression_id: attribute_value(source, reader, event, "id")?,
        type_ref: attribute_value(source, reader, event, "typeRef")?,
        text: None,
    });
    Ok(true)
}
