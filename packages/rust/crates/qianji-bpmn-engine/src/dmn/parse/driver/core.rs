//! DMN parse driver loop implementation.

use super::event::{finalize_loop_state, handle_parse_event};
use super::state::ParseLoopState;
use crate::{BpmnEngineError, DmnDecisionDefinition, DmnSourceFile};
type Result<T> = std::result::Result<T, BpmnEngineError>;
use quick_xml::Reader;
use quick_xml::events::Event;

pub(crate) fn parse_dmn_decisions_impl(
    source: &DmnSourceFile,
) -> Result<Vec<DmnDecisionDefinition>> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(true);

    let mut saw_root = false;
    let mut state = ParseLoopState::new();

    loop {
        let event = match reader.read_event() {
            Ok(event) => event,
            Err(error) => {
                return Err(BpmnEngineError::InvalidDmnXml {
                    source_id: source.source_id.clone(),
                    message: error.to_string(),
                });
            }
        };
        if matches!(event, Event::Eof) {
            break;
        }
        handle_parse_event(source, &reader, event, &mut saw_root, &mut state)?;
    }

    if !saw_root {
        return Err(BpmnEngineError::MissingDmnRootElement {
            source_id: source.source_id.clone(),
        });
    }

    finalize_loop_state(source, state)
}

pub(crate) fn parse_dmn_decision_impl(source: &DmnSourceFile) -> Result<DmnDecisionDefinition> {
    let mut decisions = parse_dmn_decisions_impl(source)?;
    if decisions.len() != 1 {
        return Err(BpmnEngineError::UnsupportedDmnDecisionCount {
            source_id: source.source_id.clone(),
            count: decisions.len(),
        });
    }
    Ok(decisions.remove(0))
}
