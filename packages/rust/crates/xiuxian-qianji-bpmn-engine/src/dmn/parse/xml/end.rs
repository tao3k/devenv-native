use crate::dmn_model_api::DmnSourceFile;
use crate::dmn_parse_api::parser::state::{
    CaptureTarget, TempContextEntry, TempContextExpression, TempDecision, TempInput,
    TempInvocation, TempInvocationBinding, TempListExpression, TempLiteralExpression, TempOutput,
    TempRelationExpression, TempRelationRow, TempRule, TempTable, finalize_input,
    finalize_input_entry, finalize_output, finalize_output_entry, finalize_rule,
};
use crate::error::Result;

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_end_tag(
    source: &DmnSourceFile,
    tag: &str,
    decisions: &mut Vec<TempDecision>,
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
) -> Result<()> {
    match tag {
        "inputExpression" if capture_target == &Some(CaptureTarget::InputExpression) => {
            if let Some(input) = current_input.as_mut() {
                let text = capture_buffer.trim();
                if !text.is_empty() {
                    input.expression = Some(text.to_string());
                }
            }
            *capture_target = None;
            capture_buffer.clear();
        }
        "text" if capture_target == &Some(CaptureTarget::LiteralExpression) => {
            if let Some(literal_expression) = current_literal.as_mut() {
                let text = capture_buffer.trim();
                if !text.is_empty() {
                    literal_expression.text = Some(text.to_string());
                }
            }
            *capture_target = None;
            capture_buffer.clear();
        }
        "description" if capture_target == &Some(CaptureTarget::RuleDescription) => {
            if let Some(rule) = current_rule.as_mut() {
                let text = capture_buffer.trim();
                if !text.is_empty() {
                    rule.description = Some(text.to_string());
                }
            }
            *capture_target = None;
            capture_buffer.clear();
        }
        "inputEntry" if capture_target == &Some(CaptureTarget::InputEntry) => {
            finalize_input_entry(source, current_rule, capture_buffer)?;
            *capture_target = None;
            capture_buffer.clear();
        }
        "outputEntry" if capture_target == &Some(CaptureTarget::OutputEntry) => {
            finalize_output_entry(source, current_rule, capture_buffer)?;
            *capture_target = None;
            capture_buffer.clear();
        }
        _ => {
            return handle_structural_end_tag(
                source,
                tag,
                StructuralEndScope {
                    decisions,
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
                    current_input,
                    current_output,
                    current_rule,
                },
            );
        }
    }
    Ok(())
}

struct StructuralEndScope<'a> {
    decisions: &'a mut Vec<TempDecision>,
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
    current_input: &'a mut Option<TempInput>,
    current_output: &'a mut Option<TempOutput>,
    current_rule: &'a mut Option<TempRule>,
}

fn handle_structural_end_tag(
    source: &DmnSourceFile,
    tag: &str,
    scope: StructuralEndScope<'_>,
) -> Result<()> {
    let StructuralEndScope {
        decisions,
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
        current_input,
        current_output,
        current_rule,
    } = scope;
    match tag {
        "input" => finalize_input(current_table, current_input),
        "output" => finalize_output(current_table, current_output),
        "rule" => finalize_rule(source, current_table, current_rule)?,
        "decisionTable" => finish_decision_table(current_decision, current_table),
        "literalExpression" => finish_literal_expression(
            current_decision,
            current_literal,
            current_list,
            current_context_entry,
            current_relation_row,
            current_invocation,
            current_invocation_binding,
        ),
        "binding" => finish_invocation_binding(current_invocation, current_invocation_binding),
        "invocation" => finish_invocation(current_decision, current_invocation),
        "contextEntry" => finish_context_entry(current_context, current_context_entry),
        "context" => finish_context(current_decision, current_context),
        "row" => finish_relation_row(current_relation, current_relation_row),
        "relation" => finish_relation(current_decision, current_relation),
        "list" => finish_list(current_decision, current_list),
        "decision" => {
            if let Some(decision) = current_decision.take() {
                decisions.push(decision);
            }
        }
        _ => {}
    }
    Ok(())
}

fn finish_decision_table(
    current_decision: &mut Option<TempDecision>,
    current_table: &mut Option<TempTable>,
) {
    let Some(table) = current_table.take() else {
        return;
    };
    let Some(decision) = current_decision.as_mut() else {
        return;
    };
    decision.table = Some(table);
}

fn finish_literal_expression(
    current_decision: &mut Option<TempDecision>,
    current_literal: &mut Option<TempLiteralExpression>,
    current_list: &mut Option<TempListExpression>,
    current_context_entry: &mut Option<TempContextEntry>,
    current_relation_row: &mut Option<TempRelationRow>,
    current_invocation: &mut Option<TempInvocation>,
    current_invocation_binding: &mut Option<TempInvocationBinding>,
) {
    let Some(literal_expression) = current_literal.take() else {
        return;
    };
    if let Some(context_entry) = current_context_entry.as_mut() {
        context_entry.literal_expression = Some(literal_expression);
        return;
    }
    if let Some(relation_row) = current_relation_row.as_mut() {
        relation_row.cells.push(literal_expression);
        return;
    }
    if let Some(list) = current_list.as_mut() {
        list.items.push(literal_expression);
        return;
    }
    if let Some(binding) = current_invocation_binding.as_mut() {
        binding.argument = Some(literal_expression);
        return;
    }
    if let Some(invocation) = current_invocation.as_mut() {
        invocation.invoked_expression = Some(literal_expression);
        return;
    }
    let Some(decision) = current_decision.as_mut() else {
        return;
    };
    decision.literal_expression = Some(literal_expression);
}

fn finish_invocation_binding(
    current_invocation: &mut Option<TempInvocation>,
    current_invocation_binding: &mut Option<TempInvocationBinding>,
) {
    let Some(binding) = current_invocation_binding.take() else {
        return;
    };
    let Some(invocation) = current_invocation.as_mut() else {
        return;
    };
    invocation.bindings.push(binding);
}

fn finish_invocation(
    current_decision: &mut Option<TempDecision>,
    current_invocation: &mut Option<TempInvocation>,
) {
    let Some(invocation) = current_invocation.take() else {
        return;
    };
    let Some(decision) = current_decision.as_mut() else {
        return;
    };
    decision.invocation = Some(invocation);
}

fn finish_context_entry(
    current_context: &mut Option<TempContextExpression>,
    current_context_entry: &mut Option<TempContextEntry>,
) {
    let Some(context_entry) = current_context_entry.take() else {
        return;
    };
    let Some(context) = current_context.as_mut() else {
        return;
    };
    context.entries.push(context_entry);
}

fn finish_context(
    current_decision: &mut Option<TempDecision>,
    current_context: &mut Option<TempContextExpression>,
) {
    let Some(context_expression) = current_context.take() else {
        return;
    };
    let Some(decision) = current_decision.as_mut() else {
        return;
    };
    decision.context_expression = Some(context_expression);
}

fn finish_relation_row(
    current_relation: &mut Option<TempRelationExpression>,
    current_relation_row: &mut Option<TempRelationRow>,
) {
    let Some(row) = current_relation_row.take() else {
        return;
    };
    let Some(relation) = current_relation.as_mut() else {
        return;
    };
    relation.rows.push(row);
}

fn finish_relation(
    current_decision: &mut Option<TempDecision>,
    current_relation: &mut Option<TempRelationExpression>,
) {
    let Some(relation_expression) = current_relation.take() else {
        return;
    };
    let Some(decision) = current_decision.as_mut() else {
        return;
    };
    decision.relation_expression = Some(relation_expression);
}

fn finish_list(
    current_decision: &mut Option<TempDecision>,
    current_list: &mut Option<TempListExpression>,
) {
    let Some(list_expression) = current_list.take() else {
        return;
    };
    let Some(decision) = current_decision.as_mut() else {
        return;
    };
    decision.list_expression = Some(list_expression);
}
