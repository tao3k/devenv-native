//! Bounded DMN XML parsing for one decision and one decision table.

#[path = "../dmn_parse_state.rs"]
mod state;
#[path = "../dmn_parse_unary.rs"]
mod unary;
#[path = "../dmn_parse_xml.rs"]
mod xml;

use self::state::{
    CaptureTarget, TempDecision, TempInput, TempOutput, TempRule, TempTable,
    finalize_decision_definition,
};
use self::xml::{
    append_capture_reference, append_capture_text, handle_end_tag, handle_start_tag, local_name,
};
use crate::dmn_model_api::{DmnDecisionDefinition, DmnSourceFile};
use crate::error::{BpmnEngineError, Result};
use quick_xml::Reader;
use quick_xml::events::Event;

pub(crate) fn parse_dmn_decision_impl(source: &DmnSourceFile) -> Result<DmnDecisionDefinition> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(true);

    let mut saw_root = false;
    let mut decision: Option<TempDecision> = None;
    let mut current_table: Option<TempTable> = None;
    let mut current_input: Option<TempInput> = None;
    let mut current_output: Option<TempOutput> = None;
    let mut current_rule: Option<TempRule> = None;
    let mut capture_target: Option<CaptureTarget> = None;
    let mut capture_buffer = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                saw_root = true;
                handle_start_tag(
                    source,
                    &reader,
                    &event,
                    &mut decision,
                    &mut current_table,
                    &mut current_input,
                    &mut current_output,
                    &mut current_rule,
                    &mut capture_target,
                    &mut capture_buffer,
                    false,
                )?;
            }
            Ok(Event::Empty(event)) => {
                saw_root = true;
                handle_start_tag(
                    source,
                    &reader,
                    &event,
                    &mut decision,
                    &mut current_table,
                    &mut current_input,
                    &mut current_output,
                    &mut current_rule,
                    &mut capture_target,
                    &mut capture_buffer,
                    true,
                )?;
            }
            Ok(Event::Text(event)) => append_capture_text(
                source,
                capture_target.as_ref(),
                &mut capture_buffer,
                event.decode(),
            )?,
            Ok(Event::CData(event)) => append_capture_text(
                source,
                capture_target.as_ref(),
                &mut capture_buffer,
                event.decode(),
            )?,
            Ok(Event::GeneralRef(event)) => append_capture_reference(
                source,
                capture_target.as_ref(),
                &mut capture_buffer,
                &event,
            )?,
            Ok(Event::End(event)) => {
                handle_end_tag(
                    source,
                    local_name(event.name().as_ref()),
                    &mut decision,
                    &mut current_table,
                    &mut current_input,
                    &mut current_output,
                    &mut current_rule,
                    &mut capture_target,
                    &mut capture_buffer,
                )?;
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => {
                return Err(BpmnEngineError::InvalidDmnXml {
                    source_id: source.source_id.clone(),
                    message: error.to_string(),
                });
            }
        }
    }

    if !saw_root {
        return Err(BpmnEngineError::MissingDmnRootElement {
            source_id: source.source_id.clone(),
        });
    }

    finalize_decision_definition(source, decision)
}
