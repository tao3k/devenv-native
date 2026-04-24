use super::decode::{attribute_value, local_name, required_attribute};
use crate::dmn_model_api::DmnSourceFile;
use crate::dmn_parse_api::parser::state::{
    CaptureTarget, TempContextEntry, TempContextExpression, TempDecision,
    TempInformationRequirementReference, TempInput, TempInvocation, TempInvocationBinding,
    TempInvocationParameter, TempKnowledgeRequirementReference, TempListExpression,
    TempLiteralExpression, TempOutput, TempRelationColumn, TempRelationExpression, TempRelationRow,
    TempRule, TempTable, finalize_input, finalize_input_entry, finalize_output,
    finalize_output_entry, finalize_rule, hit_policy_from_attr,
};
use crate::error::{BpmnEngineError, Result};
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_start_tag(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    current_decision: &mut Option<TempDecision>,
    current_literal: &mut Option<TempLiteralExpression>,
    current_list: &mut Option<TempListExpression>,
    current_context: &mut Option<TempContextExpression>,
    current_context_entry: &mut Option<TempContextEntry>,
    current_relation: &mut Option<TempRelationExpression>,
    current_relation_row: &mut Option<TempRelationRow>,
    current_invocation: &mut Option<TempInvocation>,
    current_invocation_binding: &mut Option<TempInvocationBinding>,
    current_table: &mut Option<TempTable>,
    current_input: &mut Option<TempInput>,
    current_output: &mut Option<TempOutput>,
    current_rule: &mut Option<TempRule>,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
    parent_tag: Option<&str>,
    is_empty: bool,
) -> Result<()> {
    let event_name = event.name();
    let tag = local_name(event_name.as_ref());
    if handle_decision_start_tag(
        source,
        reader,
        event,
        tag,
        DecisionStartScope {
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
        },
    )? {
        return Ok(());
    }
    if handle_table_start_tag(
        source,
        reader,
        event,
        tag,
        current_table,
        current_input,
        current_output,
        current_rule,
        is_empty,
    )? {
        return Ok(());
    }
    if handle_input_expression_start_tag(
        source,
        reader,
        event,
        tag,
        current_input,
        capture_target,
        capture_buffer,
    )? {
        return Ok(());
    }
    if handle_literal_expression_text_start_tag(
        tag,
        current_literal.as_ref(),
        capture_target,
        capture_buffer,
    ) {
        return Ok(());
    }
    handle_capture_start_tag(
        source,
        tag,
        current_rule,
        capture_target,
        capture_buffer,
        is_empty,
    )
}

struct DecisionStartScope<'a> {
    current_decision: &'a mut Option<TempDecision>,
    current_literal: &'a mut Option<TempLiteralExpression>,
    current_list: &'a mut Option<TempListExpression>,
    current_context: &'a mut Option<TempContextExpression>,
    current_context_entry: &'a mut Option<TempContextEntry>,
    current_relation: &'a mut Option<TempRelationExpression>,
    current_relation_row: &'a mut Option<TempRelationRow>,
    current_invocation: &'a mut Option<TempInvocation>,
    current_invocation_binding: &'a mut Option<TempInvocationBinding>,
    current_table: &'a mut Option<TempTable>,
    parent_tag: Option<&'a str>,
}

#[derive(Clone, Copy)]
struct SurfaceStartState<'a> {
    decision: Option<&'a TempDecision>,
    literal: Option<&'a TempLiteralExpression>,
    invocation: Option<&'a TempInvocation>,
    table: Option<&'a TempTable>,
}

impl<'a> SurfaceStartState<'a> {
    fn new(
        decision: Option<&'a TempDecision>,
        literal: Option<&'a TempLiteralExpression>,
        invocation: Option<&'a TempInvocation>,
        table: Option<&'a TempTable>,
    ) -> Self {
        Self {
            decision,
            literal,
            invocation,
            table,
        }
    }
}

#[derive(Clone, Copy)]
struct PeerSurfaceState<'a> {
    list: Option<&'a TempListExpression>,
    context: Option<&'a TempContextExpression>,
    relation: Option<&'a TempRelationExpression>,
}

impl<'a> PeerSurfaceState<'a> {
    fn new(
        list: Option<&'a TempListExpression>,
        context: Option<&'a TempContextExpression>,
        relation: Option<&'a TempRelationExpression>,
    ) -> Self {
        Self {
            list,
            context,
            relation,
        }
    }
}

#[derive(Clone, Copy)]
struct BoxedExpressionPeerState<'a> {
    list: Option<&'a TempListExpression>,
    context: Option<&'a TempContextExpression>,
    relation: Option<&'a TempRelationExpression>,
    invocation: Option<&'a TempInvocation>,
}

impl<'a> BoxedExpressionPeerState<'a> {
    fn new(
        list: Option<&'a TempListExpression>,
        context: Option<&'a TempContextExpression>,
        relation: Option<&'a TempRelationExpression>,
        invocation: Option<&'a TempInvocation>,
    ) -> Self {
        Self {
            list,
            context,
            relation,
            invocation,
        }
    }
}

fn handle_decision_start_tag(
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

struct DecisionChildStartScope<'a> {
    decision: Option<&'a TempDecision>,
    literal: &'a mut Option<TempLiteralExpression>,
    list: &'a mut Option<TempListExpression>,
    context: &'a mut Option<TempContextExpression>,
    context_entry: &'a mut Option<TempContextEntry>,
    relation: &'a mut Option<TempRelationExpression>,
    relation_row: &'a mut Option<TempRelationRow>,
    invocation: &'a mut Option<TempInvocation>,
    invocation_binding: &'a mut Option<TempInvocationBinding>,
    table: &'a mut Option<TempTable>,
}

fn handle_decision_child_start_tag(
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

fn handle_decision_requirement_reference_start_tag(
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
                    reference_kind: tag.to_string(),
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
                    reference_kind: tag.to_string(),
                    href: attribute_value(source, reader, event, "href")?,
                });
            Ok(true)
        }
        _ => Ok(false),
    }
}

struct DirectDecisionSurfaceStartScope<'a> {
    decision: Option<&'a TempDecision>,
    literal: &'a mut Option<TempLiteralExpression>,
    list: &'a mut Option<TempListExpression>,
    context: &'a mut Option<TempContextExpression>,
    relation: &'a mut Option<TempRelationExpression>,
    invocation: &'a mut Option<TempInvocation>,
    invocation_binding: &'a mut Option<TempInvocationBinding>,
    table: Option<&'a TempTable>,
}

fn handle_direct_decision_surface_start_tag(
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

struct InvocationChildStartScope<'a> {
    literal: &'a mut Option<TempLiteralExpression>,
    invocation: &'a mut Option<TempInvocation>,
    binding: &'a mut Option<TempInvocationBinding>,
}

fn handle_invocation_child_start_tag(
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

struct ContextChildStartScope<'a> {
    literal: &'a mut Option<TempLiteralExpression>,
    context: Option<&'a TempContextExpression>,
    entry: &'a mut Option<TempContextEntry>,
}

fn handle_context_child_start_tag(
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

struct RelationChildStartScope<'a> {
    literal: &'a mut Option<TempLiteralExpression>,
    relation: Option<&'a mut TempRelationExpression>,
    row: &'a mut Option<TempRelationRow>,
}

fn handle_relation_child_start_tag(
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

fn handle_list_child_start_tag(
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

fn start_decision(
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

fn start_direct_literal_expression(
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

fn start_list_expression(
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

fn start_context_expression(
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

fn start_relation_expression(
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

fn start_relation_column(
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

fn start_relation_row(
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

fn start_relation_literal_expression(
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

fn start_context_entry(
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

fn start_context_variable(
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

fn start_context_literal_expression(
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

fn start_list_literal_expression(
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

fn start_invocation_expression(
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

fn start_invocation_binding(
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

fn start_invocation_parameter(
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

fn start_invocation_literal_expression(
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

fn start_invocation_binding_argument(
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

fn start_decision_table(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    surface: SurfaceStartState<'_>,
    current_table: &mut Option<TempTable>,
    peers: PeerSurfaceState<'_>,
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
        || surface.literal.is_some()
        || surface.invocation.is_some()
        || surface.table.is_some()
        || current_table.is_some()
        || peers.list.is_some()
        || peers.context.is_some()
        || peers.relation.is_some()
    {
        return Err(BpmnEngineError::UnsupportedDmnDecisionTableCount {
            decision_id: decision.decision_id.clone(),
            count: 2,
        });
    }
    *current_table = Some(TempTable {
        table_id: required_attribute(source, reader, event, "decisionTable", "id")?,
        name: attribute_value(source, reader, event, "name")?,
        hit_policy: hit_policy_from_attr(
            source,
            decision.decision_id.as_str(),
            attribute_value(source, reader, event, "hitPolicy")?.as_deref(),
        )?,
        inputs: Vec::new(),
        outputs: Vec::new(),
        rules: Vec::new(),
    });
    Ok(true)
}

fn handle_literal_expression_text_start_tag(
    tag: &str,
    current_literal: Option<&TempLiteralExpression>,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
) -> bool {
    if tag != "text" || current_literal.is_none() || capture_target.is_some() {
        return false;
    }
    *capture_target = Some(CaptureTarget::LiteralExpression);
    capture_buffer.clear();
    true
}

#[allow(clippy::too_many_arguments)]
fn handle_table_start_tag(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    current_table: &mut Option<TempTable>,
    current_input: &mut Option<TempInput>,
    current_output: &mut Option<TempOutput>,
    current_rule: &mut Option<TempRule>,
    is_empty: bool,
) -> Result<bool> {
    match tag {
        "input" => {
            if current_table.is_none() {
                return Ok(true);
            }
            *current_input = Some(TempInput {
                input_id: required_attribute(source, reader, event, "input", "id")?,
                label: attribute_value(source, reader, event, "label")?,
                name: attribute_value(source, reader, event, "name")?,
                expression: None,
                type_ref: None,
            });
            if is_empty {
                finalize_input(current_table, current_input);
            }
            Ok(true)
        }
        "output" => {
            if current_table.is_none() {
                return Ok(true);
            }
            *current_output = Some(TempOutput {
                output_id: required_attribute(source, reader, event, "output", "id")?,
                label: attribute_value(source, reader, event, "label")?,
                name: attribute_value(source, reader, event, "name")?,
                type_ref: attribute_value(source, reader, event, "typeRef")?,
            });
            if is_empty {
                finalize_output(current_table, current_output);
            }
            Ok(true)
        }
        "rule" => {
            if current_table.is_none() {
                return Ok(true);
            }
            *current_rule = Some(TempRule {
                rule_id: required_attribute(source, reader, event, "rule", "id")?,
                description: None,
                input_entries: Vec::new(),
                output_entries: Vec::new(),
            });
            if is_empty {
                finalize_rule(source, current_table, current_rule)?;
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn handle_input_expression_start_tag(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    current_input: &mut Option<TempInput>,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
) -> Result<bool> {
    if tag != "inputExpression" {
        return Ok(false);
    }
    if let Some(input) = current_input.as_mut() {
        input.type_ref = attribute_value(source, reader, event, "typeRef")?;
    }
    *capture_target = Some(CaptureTarget::InputExpression);
    capture_buffer.clear();
    Ok(true)
}

fn handle_capture_start_tag(
    source: &DmnSourceFile,
    tag: &str,
    current_rule: &mut Option<TempRule>,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
    is_empty: bool,
) -> Result<()> {
    match tag {
        "description" if current_rule.is_some() => {
            *capture_target = Some(CaptureTarget::RuleDescription);
            capture_buffer.clear();
        }
        "inputEntry" => {
            *capture_target = Some(CaptureTarget::InputEntry);
            capture_buffer.clear();
            if is_empty {
                finalize_input_entry(source, current_rule, capture_buffer)?;
                *capture_target = None;
                capture_buffer.clear();
            }
        }
        "outputEntry" => {
            *capture_target = Some(CaptureTarget::OutputEntry);
            capture_buffer.clear();
            if is_empty {
                finalize_output_entry(source, current_rule, capture_buffer)?;
                *capture_target = None;
                capture_buffer.clear();
            }
        }
        _ => {}
    }
    Ok(())
}
