use crate::dmn_model_api::DmnSourceFile;
use crate::dmn_parse_api::parser::state::{
    CaptureTarget, TempDecision, TempInput, TempOutput, TempRule, TempTable, finalize_input,
    finalize_input_entry, finalize_output, finalize_output_entry, finalize_rule,
};
use crate::error::Result;

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_end_tag(
    source: &DmnSourceFile,
    tag: &str,
    decisions: &mut Vec<TempDecision>,
    current_decision: &mut Option<TempDecision>,
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
        "input" => finalize_input(current_table, current_input),
        "output" => finalize_output(current_table, current_output),
        "rule" => finalize_rule(source, current_table, current_rule)?,
        "decisionTable" => {
            let Some(table) = current_table.take() else {
                return Ok(());
            };
            let Some(decision) = current_decision.as_mut() else {
                return Ok(());
            };
            decision.table = Some(table);
        }
        "decision" => {
            if let Some(decision) = current_decision.take() {
                decisions.push(decision);
            }
        }
        _ => {}
    }
    Ok(())
}
