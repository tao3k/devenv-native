use super::{
    BoxedExpressionPeerState, BpmnEngineError, BytesStart, ContextChildStartScope, DmnSourceFile,
    Reader, Result, SurfaceStartState, TempContextEntry, TempContextExpression,
    TempLiteralExpression, attribute_value,
};

pub(super) fn handle_context_child_start_tag(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    parent_tag: Option<&str>,
    scope: ContextChildStartScope<'_>,
) -> Result<Option<bool>> {
    let ContextChildStartScope {
        literal,
        context,
        entry,
    } = scope;
    match parent_tag {
        Some("context") => match tag {
            "contextEntry" => start_context_entry(source, reader, event, context, entry).map(Some),
            _ if context.is_some() => Err(BpmnEngineError::UnsupportedOperation {
                operation: "parse_dmn_context_unsupported_child",
            }),
            _ => Ok(Some(false)),
        },
        Some("contextEntry") => match tag {
            "variable" => start_context_variable(source, reader, event, entry).map(Some),
            "literalExpression" => {
                start_context_literal_expression(source, reader, event, literal, entry.as_ref())
                    .map(Some)
            }
            _ if entry.is_some() => Err(BpmnEngineError::UnsupportedOperation {
                operation: "parse_dmn_context_unsupported_child",
            }),
            _ => Ok(Some(false)),
        },
        _ => Ok(None),
    }
}

pub(super) fn start_context_expression(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    surface: SurfaceStartState<'_>,
    current_context: &mut Option<TempContextExpression>,
    peers: BoxedExpressionPeerState<'_>,
) -> Result<bool> {
    let Some(decision) = surface.decision else {
        return Ok(true);
    };
    if decision.table.is_some()
        || decision.literal_expression.is_some()
        || decision.list_expression.is_some()
        || decision.context_expression.is_some()
        || decision.invocation.is_some()
        || surface.table.is_some()
        || surface.literal.is_some()
        || peers.list.is_some()
        || current_context.is_some()
        || peers.invocation.is_some()
        || peers.relation.is_some()
    {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_context_mixed_with_other_executable_surface",
        });
    }
    *current_context = Some(TempContextExpression {
        context_id: attribute_value(source, reader, event, "id")?,
        entries: Vec::new(),
    });
    Ok(true)
}

pub(super) fn start_context_entry(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    current_context: Option<&TempContextExpression>,
    current_context_entry: &mut Option<TempContextEntry>,
) -> Result<bool> {
    if current_context.is_none() {
        return Ok(true);
    }
    if current_context_entry.is_some() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_context_nested_entry",
        });
    }
    *current_context_entry = Some(TempContextEntry {
        entry_id: attribute_value(source, reader, event, "id")?,
        variable_id: None,
        variable_name: None,
        literal_expression: None,
    });
    Ok(true)
}

pub(super) fn start_context_variable(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    current_context_entry: &mut Option<TempContextEntry>,
) -> Result<bool> {
    let Some(entry) = current_context_entry.as_mut() else {
        return Ok(true);
    };
    entry.variable_id = attribute_value(source, reader, event, "id")?;
    entry.variable_name = attribute_value(source, reader, event, "name")?;
    Ok(true)
}

pub(super) fn start_context_literal_expression(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    current_literal: &mut Option<TempLiteralExpression>,
    current_context_entry: Option<&TempContextEntry>,
) -> Result<bool> {
    if current_context_entry.is_none() {
        return Ok(true);
    }
    if current_literal.is_some() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_context_nested_literal_expression",
        });
    }
    *current_literal = Some(TempLiteralExpression {
        expression_id: attribute_value(source, reader, event, "id")?,
        type_ref: attribute_value(source, reader, event, "typeRef")?,
        text: None,
    });
    Ok(true)
}
