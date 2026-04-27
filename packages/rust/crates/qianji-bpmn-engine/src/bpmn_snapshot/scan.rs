use super::state::{BpmnSnapshotScanState, TextTarget};
use super::xml::{append_reference_content, append_text_content, local_name};
use crate::bpmn_model_api::BpmnDocumentSnapshot;
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use quick_xml::Reader;
use quick_xml::events::Event;

pub(crate) fn snapshot_bpmn_source_sync(source: &BpmnSourceFile) -> Result<BpmnDocumentSnapshot> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(true);

    let mut saw_root = false;
    let mut state = BpmnSnapshotScanState::new();
    let mut element_stack = Vec::new();

    loop {
        let event = match reader.read_event() {
            Ok(event) => event,
            Err(error) => {
                return Err(BpmnEngineError::InvalidXml {
                    source_id: source.source_id.clone(),
                    message: error.to_string(),
                    offset: Some(reader.error_position()),
                });
            }
        };
        if handle_scan_event(
            source,
            &reader,
            event,
            &mut saw_root,
            &mut state,
            &mut element_stack,
        )? {
            break;
        }
    }

    ensure_root_seen(source, saw_root)?;
    state.finish_pending();
    Ok(state.into_snapshot(source))
}

fn handle_scan_event(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: Event<'_>,
    saw_root: &mut bool,
    state: &mut BpmnSnapshotScanState,
    element_stack: &mut Vec<String>,
) -> Result<bool> {
    match event {
        Event::Start(event) => {
            *saw_root = true;
            let tag = local_name(event.name().as_ref()).to_string();
            state.handle_start_event(
                source,
                reader,
                &event,
                element_stack.last().map(String::as_str),
                false,
            )?;
            element_stack.push(tag);
            Ok(false)
        }
        Event::Empty(event) => {
            *saw_root = true;
            state.handle_start_event(
                source,
                reader,
                &event,
                element_stack.last().map(String::as_str),
                true,
            )?;
            Ok(false)
        }
        Event::End(event) => {
            let event_name = event.name();
            let tag = local_name(event_name.as_ref());
            state.finish_end_event(tag);
            if element_stack.last().is_some_and(|open_tag| open_tag == tag) {
                element_stack.pop();
            }
            Ok(false)
        }
        Event::Text(event) => {
            handle_text_event(state, element_stack, |buffer| {
                append_text_content(source, buffer, event.decode())
            })?;
            Ok(false)
        }
        Event::CData(event) => {
            handle_text_event(state, element_stack, |buffer| {
                append_text_content(source, buffer, event.decode())
            })?;
            Ok(false)
        }
        Event::GeneralRef(event) => {
            handle_text_event(state, element_stack, |buffer| {
                append_reference_content(source, buffer, &event)
            })?;
            Ok(false)
        }
        Event::Eof => Ok(true),
        Event::Decl(_) | Event::PI(_) | Event::DocType(_) | Event::Comment(_) => Ok(false),
    }
}

fn handle_text_event<F>(
    state: &mut BpmnSnapshotScanState,
    element_stack: &[String],
    append_chunk: F,
) -> Result<()>
where
    F: FnOnce(&mut String) -> Result<()>,
{
    let mut text = String::new();
    append_chunk(&mut text)?;
    state.handle_text_chunk(&text, text_target(element_stack));
    Ok(())
}

fn text_target(element_stack: &[String]) -> Option<TextTarget> {
    match (
        element_stack.last().map(String::as_str),
        parent_tag(element_stack),
    ) {
        (Some("flowNodeRef"), Some("lane")) => Some(TextTarget::LaneFlowNode),
        (Some("sourceRef"), Some("dataInputAssociation" | "dataOutputAssociation")) => {
            Some(TextTarget::DataAssociationSource)
        }
        (Some("targetRef"), Some("dataInputAssociation" | "dataOutputAssociation")) => {
            Some(TextTarget::DataAssociationTarget)
        }
        _ => None,
    }
}

fn parent_tag(element_stack: &[String]) -> Option<&str> {
    element_stack
        .len()
        .checked_sub(2)
        .and_then(|index| element_stack.get(index))
        .map(String::as_str)
}

fn ensure_root_seen(source: &BpmnSourceFile, saw_root: bool) -> Result<()> {
    if !saw_root {
        return Err(BpmnEngineError::MissingRootElement {
            source_id: source.source_id.clone(),
        });
    }
    Ok(())
}
