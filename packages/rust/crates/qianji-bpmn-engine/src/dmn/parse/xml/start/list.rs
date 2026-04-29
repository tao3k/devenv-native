use super::{
    BoxedExpressionPeerState, BpmnEngineError, BytesStart, DmnSourceFile, Reader, Result,
    SurfaceStartState, TempListExpression, TempLiteralExpression, attribute_value,
};

pub(super) fn handle_list_child_start_tag(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    parent_tag: Option<&str>,
    current_literal: &mut Option<TempLiteralExpression>,
    current_list: Option<&TempListExpression>,
) -> Result<Option<bool>> {
    if parent_tag != Some("list") {
        return Ok(None);
    }
    match tag {
        "literalExpression" => {
            start_list_literal_expression(source, reader, event, current_literal, current_list)
                .map(Some)
        }
        _ if current_list.is_some() => Err(BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_list_unsupported_child",
        }),
        _ => Ok(Some(false)),
    }
}

pub(super) fn start_list_expression(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    surface: SurfaceStartState<'_>,
    current_list: &mut Option<TempListExpression>,
    peers: BoxedExpressionPeerState<'_>,
) -> Result<bool> {
    let Some(decision) = surface.decision else {
        return Ok(true);
    };
    if decision.table.is_some()
        || decision.literal_expression.is_some()
        || decision.list_expression.is_some()
        || decision.invocation.is_some()
        || surface.table.is_some()
        || surface.literal.is_some()
        || surface.invocation.is_some()
        || current_list.is_some()
        || peers.context.is_some()
        || peers.invocation.is_some()
        || peers.relation.is_some()
    {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_list_mixed_with_other_executable_surface",
        });
    }
    *current_list = Some(TempListExpression {
        list_id: attribute_value(source, reader, event, "id")?,
        items: Vec::new(),
    });
    Ok(true)
}

pub(super) fn start_list_literal_expression(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    current_literal: &mut Option<TempLiteralExpression>,
    current_list: Option<&TempListExpression>,
) -> Result<bool> {
    if current_list.is_none() {
        return Ok(true);
    }
    *current_literal = Some(TempLiteralExpression {
        expression_id: attribute_value(source, reader, event, "id")?,
        type_ref: attribute_value(source, reader, event, "typeRef")?,
        text: None,
    });
    Ok(true)
}
