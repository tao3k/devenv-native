use super::state::ParseLoopState;
use crate::dmn_model_api::{DmnDecisionDefinition, DmnSourceFile};
use crate::dmn_parse_api::parser::state::finalize_decision_definitions;
use crate::dmn_parse_api::parser::xml::{
    append_capture_reference, append_capture_text, handle_end_tag, handle_start_tag, local_name,
    validate_dmn_root_start_tag,
};
use crate::error::{BpmnEngineError, Result};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

pub(super) fn handle_parse_event(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: Event<'_>,
    saw_root: &mut bool,
    state: &mut ParseLoopState,
) -> Result<()> {
    match event {
        Event::Start(event) => {
            let tag = local_name(event.name().as_ref()).to_string();
            handle_start_event(source, reader, &event, saw_root, state, false)?;
            state.element_stack.push(tag);
            Ok(())
        }
        Event::Empty(event) => handle_start_event(source, reader, &event, saw_root, state, true),
        Event::Text(event) => append_capture_text(
            source,
            state.capture_target.as_ref(),
            &mut state.capture_buffer,
            event.decode(),
        ),
        Event::CData(event) => append_capture_text(
            source,
            state.capture_target.as_ref(),
            &mut state.capture_buffer,
            event.decode(),
        ),
        Event::GeneralRef(event) => append_capture_reference(
            source,
            state.capture_target.as_ref(),
            &mut state.capture_buffer,
            &event,
        ),
        Event::End(event) => {
            let tag = local_name(event.name().as_ref()).to_string();
            handle_end_tag(
                source,
                tag.as_str(),
                &mut state.decisions,
                &mut state.current_decision,
                &mut state.current_literal,
                &mut state.current_list,
                &mut state.current_context,
                &mut state.current_context_entry,
                &mut state.current_relation,
                &mut state.current_relation_row,
                &mut state.current_invocation,
                &mut state.current_invocation_binding,
                &mut state.current_table,
                &mut state.current_input,
                &mut state.current_output,
                &mut state.current_rule,
                &mut state.capture_target,
                &mut state.capture_buffer,
            )?;
            if state
                .element_stack
                .last()
                .is_some_and(|open_tag| open_tag == &tag)
            {
                state.element_stack.pop();
            }
            Ok(())
        }
        Event::Eof | Event::Decl(_) | Event::PI(_) | Event::DocType(_) | Event::Comment(_) => {
            Ok(())
        }
    }
}

fn handle_start_event(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    saw_root: &mut bool,
    state: &mut ParseLoopState,
    is_empty: bool,
) -> Result<()> {
    let event_name = event.name();
    let tag = local_name(event_name.as_ref());
    let parent_tag = state.element_stack.last().map(String::as_str);

    if !*saw_root {
        validate_dmn_root_start_tag(source, reader, event)?;
        *saw_root = true;
    }

    reject_unsupported_top_level_import(source, tag, parent_tag)?;

    handle_start_tag(
        source,
        reader,
        event,
        &mut state.current_decision,
        &mut state.current_literal,
        &mut state.current_list,
        &mut state.current_context,
        &mut state.current_context_entry,
        &mut state.current_relation,
        &mut state.current_relation_row,
        &mut state.current_invocation,
        &mut state.current_invocation_binding,
        &mut state.current_table,
        &mut state.current_input,
        &mut state.current_output,
        &mut state.current_rule,
        &mut state.capture_target,
        &mut state.capture_buffer,
        parent_tag,
        is_empty,
    )
}

fn reject_unsupported_top_level_import(
    source: &DmnSourceFile,
    tag: &str,
    parent_tag: Option<&str>,
) -> Result<()> {
    if tag == "import" && parent_tag == Some("definitions") {
        return Err(BpmnEngineError::UnsupportedDmnImport {
            source_id: (source.source_id.clone()).into(),
        });
    }
    Ok(())
}

pub(super) fn finalize_loop_state(
    source: &DmnSourceFile,
    mut state: ParseLoopState,
) -> Result<Vec<DmnDecisionDefinition>> {
    if let Some(decision) = state.current_decision.take() {
        state.decisions.push(decision);
    }

    finalize_decision_definitions(source, state.decisions)
}
