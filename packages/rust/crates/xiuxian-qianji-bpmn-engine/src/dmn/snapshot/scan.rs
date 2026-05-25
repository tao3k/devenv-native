use super::root::empty_root_snapshot;
use super::state::SnapshotScanState;
use super::xml::{append_reference_content, append_text_content, local_name};
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
        let event = match reader.read_event() {
            Ok(event) => event,
            Err(error) => {
                return Err(BpmnEngineError::InvalidDmnXml {
                    source_id: (source.source_id.clone()).into(),
                    message: error.to_string(),
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
    finish_snapshot_state(&mut state);
    Ok(build_snapshot_document(source, state))
}

fn handle_scan_event(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: Event<'_>,
    saw_root: &mut bool,
    state: &mut SnapshotScanState,
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
            finish_end_event(state, element_stack, local_name(event.name().as_ref()));
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

fn finish_end_event(state: &mut SnapshotScanState, element_stack: &mut Vec<String>, tag: &str) {
    match tag {
        "decision" => state.finish_decision_end(),
        "invocation" => state.finish_invocation_end(),
        "binding" => state.finish_invocation_binding_end(),
        "functionDefinition" | "encapsulatedLogic" => state.finish_function_definition_end(),
        "literalExpression" => state.finish_literal_expression_end(),
        "businessKnowledgeModel" => state.finish_business_knowledge_model_end(),
        "decisionService" => state.finish_decision_service_end(),
        "textAnnotation" => state.finish_text_annotation_end(),
        "association" => state.finish_association_end(),
        "DMNDiagram" => state.finish_dmn_diagram_end(),
        "DMNDI" => state.finish_dmndi_end(),
        "DMNLabel" => state.finish_dmn_label_end(),
        "inputData" => state.finish_input_data_end(),
        "itemDefinition" => state.finish_item_definition_end(),
        _ => {}
    }
    if element_stack.last().is_some_and(|open_tag| open_tag == tag) {
        element_stack.pop();
    }
}

fn handle_text_event<F>(
    state: &mut SnapshotScanState,
    element_stack: &[String],
    append_chunk: F,
) -> Result<()>
where
    F: FnOnce(&mut String) -> Result<()>,
{
    let mut text = String::new();
    append_chunk(&mut text)?;
    let current_tag = element_stack.last().map(String::as_str);
    let parent_tag = parent_tag(element_stack);
    state.handle_text_chunk(&text, current_tag, parent_tag);
    Ok(())
}

fn parent_tag(element_stack: &[String]) -> Option<&str> {
    element_stack
        .len()
        .checked_sub(2)
        .and_then(|index| element_stack.get(index))
        .map(String::as_str)
}

fn ensure_root_seen(source: &DmnSourceFile, saw_root: bool) -> Result<()> {
    if !saw_root {
        return Err(BpmnEngineError::MissingDmnRootElement {
            source_id: (source.source_id.clone()).into(),
        });
    }
    Ok(())
}

fn finish_snapshot_state(state: &mut SnapshotScanState) {
    state.finish_pending_decision();
    state.finish_pending_text_annotation();
    state.finish_pending_association();
    state.finish_pending_dmndi();
    state.finish_pending_input_data();
    state.finish_pending_business_knowledge_model();
    state.finish_pending_decision_service();
    state.finish_pending_item_definition();
}

fn build_snapshot_document(
    source: &DmnSourceFile,
    state: SnapshotScanState,
) -> DmnDocumentSnapshot {
    let (root, decisions) = state.into_parts();
    DmnDocumentSnapshot {
        source_id: (source.source_id.clone()),
        root: root.unwrap_or_else(empty_root_snapshot),
        decisions,
    }
}
