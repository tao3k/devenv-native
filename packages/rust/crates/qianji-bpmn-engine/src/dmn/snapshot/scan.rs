use super::root::empty_root_snapshot;
use super::state::SnapshotScanState;
use super::xml::local_name;
use crate::dmn_model_api::{DmnDocumentSnapshot, DmnSourceFile};
use crate::error::{BpmnEngineError, Result};
use quick_xml::Reader;
use quick_xml::events::Event;

pub(crate) fn snapshot_dmn_source_sync(source: &DmnSourceFile) -> Result<DmnDocumentSnapshot> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(true);

    let mut saw_root = false;
    let mut state = SnapshotScanState::new();
    let mut element_stack = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                saw_root = true;
                let tag = local_name(event.name().as_ref()).to_string();
                state.handle_start_event(
                    source,
                    &reader,
                    &event,
                    element_stack.last().map(String::as_str),
                    false,
                )?;
                element_stack.push(tag);
            }
            Ok(Event::Empty(event)) => {
                saw_root = true;
                state.handle_start_event(
                    source,
                    &reader,
                    &event,
                    element_stack.last().map(String::as_str),
                    true,
                )?;
            }
            Ok(Event::End(event)) => {
                let tag = local_name(event.name().as_ref()).to_string();
                if tag == "decision" {
                    state.finish_decision_end();
                }
                if element_stack
                    .last()
                    .is_some_and(|open_tag| open_tag == &tag)
                {
                    element_stack.pop();
                }
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

    state.finish_pending_decision();
    let (root, decisions) = state.into_parts();

    Ok(DmnDocumentSnapshot {
        source_id: source.source_id.clone(),
        root: root.unwrap_or_else(empty_root_snapshot),
        decisions,
    })
}
