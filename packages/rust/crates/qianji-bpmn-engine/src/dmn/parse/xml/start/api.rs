use super::{
    BytesStart, CaptureTarget, DecisionStartScope, DmnSourceFile, Reader, Result, TempContextEntry,
    TempContextExpression, TempDecision, TempInput, TempInvocation, TempInvocationBinding,
    TempListExpression, TempLiteralExpression, TempOutput, TempRelationExpression, TempRelationRow,
    TempRule, TempTable, handle_capture_start_tag, handle_decision_start_tag,
    handle_input_expression_start_tag, handle_literal_expression_text_start_tag,
    handle_table_start_tag, local_name,
};

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
