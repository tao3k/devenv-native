use super::{
    BoxedExpressionPeerState, BpmnEngineError, BytesStart, ContextChildStartScope,
    DecisionChildStartScope, DecisionStartScope, DirectDecisionSurfaceStartScope, DmnSourceFile,
    InvocationChildStartScope, PeerSurfaceState, Reader, RelationChildStartScope, Result,
    SurfaceStartState, TempDecision, TempInformationRequirementReference,
    TempKnowledgeRequirementReference, TempLiteralExpression, TempTable, attribute_value,
    handle_context_child_start_tag, handle_invocation_child_start_tag, handle_list_child_start_tag,
    handle_relation_child_start_tag, required_attribute, start_context_expression,
    start_decision_table, start_invocation_expression, start_list_expression,
    start_relation_expression,
};

pub(super) fn handle_decision_start_tag(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    scope: DecisionStartScope<'_>,
) -> Result<bool> {
    let DecisionStartScope {
        current_decision,
        current_literal,
        current_list,
        current_context,
        current_context_entry,
        current_relation,
        current_relation_row,
        current_invocation,
        current_invocation_binding,
        current_table,
        parent_tag,
    } = scope;
    if tag == "decision" {
        return start_decision(source, reader, event, current_decision);
    }
    if parent_tag == Some("decision")
        && handle_direct_decision_surface_start_tag(
            source,
            reader,
            event,
            tag,
            DirectDecisionSurfaceStartScope {
                decision: current_decision.as_ref(),
                literal: current_literal,
                list: current_list,
                context: current_context,
                relation: current_relation,
                invocation: current_invocation,
                invocation_binding: current_invocation_binding,
                table: current_table.as_ref(),
            },
        )?
    {
        return Ok(true);
    }
    if handle_decision_requirement_reference_start_tag(
        source,
        reader,
        event,
        tag,
        parent_tag,
        current_decision,
    )? {
        return Ok(true);
    }
    handle_decision_child_start_tag(
        source,
        reader,
        event,
        tag,
        parent_tag,
        DecisionChildStartScope {
            decision: current_decision.as_ref(),
            literal: current_literal,
            list: current_list,
            context: current_context,
            context_entry: current_context_entry,
            relation: current_relation,
            relation_row: current_relation_row,
            invocation: current_invocation,
            invocation_binding: current_invocation_binding,
            table: current_table,
        },
    )
}

pub(super) fn handle_decision_child_start_tag(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    parent_tag: Option<&str>,
    scope: DecisionChildStartScope<'_>,
) -> Result<bool> {
    let DecisionChildStartScope {
        decision,
        literal,
        list,
        context,
        context_entry,
        relation,
        relation_row,
        invocation,
        invocation_binding,
        table,
    } = scope;
    if let Some(handled) = handle_invocation_child_start_tag(
        source,
        reader,
        event,
        tag,
        parent_tag,
        InvocationChildStartScope {
            literal,
            invocation,
            binding: invocation_binding,
        },
    )? {
        return Ok(handled);
    }
    if let Some(handled) = handle_context_child_start_tag(
        source,
        reader,
        event,
        tag,
        parent_tag,
        ContextChildStartScope {
            literal,
            context: context.as_ref(),
            entry: context_entry,
        },
    )? {
        return Ok(handled);
    }
    if let Some(handled) = handle_relation_child_start_tag(
        source,
        reader,
        event,
        tag,
        parent_tag,
        RelationChildStartScope {
            literal,
            relation: relation.as_mut(),
            row: relation_row,
        },
    )? {
        return Ok(handled);
    }
    if let Some(handled) = handle_list_child_start_tag(
        source,
        reader,
        event,
        tag,
        parent_tag,
        literal,
        list.as_ref(),
    )? {
        return Ok(handled);
    }
    if tag == "decisionTable" {
        return start_decision_table(
            source,
            reader,
            event,
            SurfaceStartState::new(decision, literal.as_ref(), invocation.as_ref(), None),
            table,
            PeerSurfaceState::new(list.as_ref(), context.as_ref(), relation.as_ref()),
        );
    }
    Ok(false)
}

pub(super) fn handle_decision_requirement_reference_start_tag(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    parent_tag: Option<&str>,
    current_decision: &mut Option<TempDecision>,
) -> Result<bool> {
    match (parent_tag, tag) {
        (Some("informationRequirement"), "requiredInput" | "requiredDecision") => {
            let Some(decision) = current_decision.as_mut() else {
                return Err(BpmnEngineError::UnsupportedOperation {
                    operation: "parse_dmn_information_requirement_without_decision",
                });
            };
            decision
                .information_requirements
                .push(TempInformationRequirementReference {
                    reference_kind: (tag.to_string()),
                    href: attribute_value(source, reader, event, "href")?,
                });
            Ok(true)
        }
        (Some("knowledgeRequirement"), "requiredKnowledge") => {
            let Some(decision) = current_decision.as_mut() else {
                return Err(BpmnEngineError::UnsupportedOperation {
                    operation: "parse_dmn_knowledge_requirement_without_decision",
                });
            };
            decision
                .knowledge_requirements
                .push(TempKnowledgeRequirementReference {
                    reference_kind: (tag.to_string()),
                    href: attribute_value(source, reader, event, "href")?,
                });
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(super) fn handle_direct_decision_surface_start_tag(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    scope: DirectDecisionSurfaceStartScope<'_>,
) -> Result<bool> {
    let DirectDecisionSurfaceStartScope {
        decision,
        literal,
        list,
        context,
        relation,
        invocation,
        invocation_binding,
        table,
    } = scope;
    match tag {
        "literalExpression" => {
            start_direct_literal_expression(source, reader, event, decision, literal, table)
        }
        "list" => start_list_expression(
            source,
            reader,
            event,
            SurfaceStartState::new(decision, literal.as_ref(), invocation.as_ref(), table),
            list,
            BoxedExpressionPeerState::new(
                None,
                context.as_ref(),
                relation.as_ref(),
                invocation.as_ref(),
            ),
        ),
        "invocation" => start_invocation_expression(
            source,
            reader,
            event,
            SurfaceStartState::new(decision, literal.as_ref(), None, table),
            invocation,
            invocation_binding.as_ref(),
            BoxedExpressionPeerState::new(list.as_ref(), context.as_ref(), relation.as_ref(), None),
        ),
        "context" => start_context_expression(
            source,
            reader,
            event,
            SurfaceStartState::new(decision, literal.as_ref(), invocation.as_ref(), table),
            context,
            BoxedExpressionPeerState::new(
                list.as_ref(),
                None,
                relation.as_ref(),
                invocation.as_ref(),
            ),
        ),
        "relation" => start_relation_expression(
            source,
            reader,
            event,
            SurfaceStartState::new(decision, literal.as_ref(), invocation.as_ref(), table),
            relation,
            BoxedExpressionPeerState::new(
                list.as_ref(),
                context.as_ref(),
                None,
                invocation.as_ref(),
            ),
        ),
        _ => Ok(false),
    }
}

pub(super) fn start_decision(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    current_decision: &mut Option<TempDecision>,
) -> Result<bool> {
    *current_decision = Some(TempDecision {
        decision_id: required_attribute(source, reader, event, "decision", "id")?,
        name: attribute_value(source, reader, event, "name")?,
        table: None,
        literal_expression: None,
        list_expression: None,
        context_expression: None,
        relation_expression: None,
        invocation: None,
        information_requirements: Vec::new(),
        knowledge_requirements: Vec::new(),
    });
    Ok(true)
}

pub(super) fn start_direct_literal_expression(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    current_decision: Option<&TempDecision>,
    current_literal: &mut Option<TempLiteralExpression>,
    current_table: Option<&TempTable>,
) -> Result<bool> {
    let Some(decision) = current_decision else {
        return Ok(true);
    };
    if decision.table.is_some() || current_table.is_some() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_literal_expression_mixed_with_decision_table",
        });
    }
    *current_literal = Some(TempLiteralExpression {
        expression_id: attribute_value(source, reader, event, "id")?,
        type_ref: attribute_value(source, reader, event, "typeRef")?,
        text: None,
    });
    Ok(true)
}
