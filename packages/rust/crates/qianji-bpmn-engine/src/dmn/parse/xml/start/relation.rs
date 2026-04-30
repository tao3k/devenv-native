use super::{
    BoxedExpressionPeerState, BpmnEngineError, BytesStart, DmnSourceFile, Reader,
    RelationChildStartScope, Result, SurfaceStartState, TempLiteralExpression, TempRelationColumn,
    TempRelationExpression, TempRelationRow, attribute_value, required_attribute,
};

pub(super) fn handle_relation_child_start_tag(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    parent_tag: Option<&str>,
    scope: RelationChildStartScope<'_>,
) -> Result<Option<bool>> {
    let RelationChildStartScope {
        literal,
        relation,
        row,
    } = scope;
    match parent_tag {
        Some("relation") => match tag {
            "column" => start_relation_column(source, reader, event, relation).map(Some),
            "row" => start_relation_row(source, reader, event, relation.as_deref(), row).map(Some),
            _ if relation.is_some() => Err(BpmnEngineError::UnsupportedOperation {
                operation: "parse_dmn_relation_unsupported_child",
            }),
            _ => Ok(Some(false)),
        },
        Some("row") => match tag {
            "literalExpression" => {
                start_relation_literal_expression(source, reader, event, literal, row.as_ref())
                    .map(Some)
            }
            _ if row.is_some() => Err(BpmnEngineError::UnsupportedOperation {
                operation: "parse_dmn_relation_unsupported_child",
            }),
            _ => Ok(Some(false)),
        },
        _ => Ok(None),
    }
}

pub(super) fn start_relation_expression(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    surface: SurfaceStartState<'_>,
    current_relation: &mut Option<TempRelationExpression>,
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
        || peers.list.is_some()
        || peers.context.is_some()
        || peers.invocation.is_some()
        || current_relation.is_some()
    {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_relation_mixed_with_other_executable_surface",
        });
    }
    *current_relation = Some(TempRelationExpression {
        relation_id: attribute_value(source, reader, event, "id")?,
        columns: Vec::new(),
        rows: Vec::new(),
    });
    Ok(true)
}

pub(super) fn start_relation_column(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    current_relation: Option<&mut TempRelationExpression>,
) -> Result<bool> {
    let Some(relation) = current_relation else {
        return Ok(true);
    };
    relation.columns.push(TempRelationColumn {
        column_id: required_attribute(source, reader, event, "column", "id")?,
        name: attribute_value(source, reader, event, "name")?,
        type_ref: attribute_value(source, reader, event, "typeRef")?,
    });
    Ok(true)
}

pub(super) fn start_relation_row(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    current_relation: Option<&TempRelationExpression>,
    current_relation_row: &mut Option<TempRelationRow>,
) -> Result<bool> {
    if current_relation.is_none() {
        return Ok(true);
    }
    if current_relation_row.is_some() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_relation_nested_row",
        });
    }
    *current_relation_row = Some(TempRelationRow {
        row_id: attribute_value(source, reader, event, "id")?,
        cells: Vec::new(),
    });
    Ok(true)
}

pub(super) fn start_relation_literal_expression(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    current_literal: &mut Option<TempLiteralExpression>,
    current_relation_row: Option<&TempRelationRow>,
) -> Result<bool> {
    if current_relation_row.is_none() {
        return Ok(true);
    }
    if current_literal.is_some() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_relation_nested_literal_expression",
        });
    }
    *current_literal = Some(TempLiteralExpression {
        expression_id: attribute_value(source, reader, event, "id")?,
        type_ref: attribute_value(source, reader, event, "typeRef")?,
        text: None,
    });
    Ok(true)
}
