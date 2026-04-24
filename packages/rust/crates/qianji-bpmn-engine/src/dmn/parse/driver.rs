use crate::dmn_model_api::{DmnDecisionDefinition, DmnSourceFile};
use crate::dmn_parse_api::parser::state::{
    CaptureTarget, TempContextEntry, TempContextExpression, TempDecision, TempInput,
    TempInvocation, TempInvocationBinding, TempListExpression, TempLiteralExpression, TempOutput,
    TempRelationExpression, TempRelationRow, TempRule, TempTable, finalize_decision_definitions,
};
use crate::dmn_parse_api::parser::xml::{
    append_capture_reference, append_capture_text, handle_end_tag, handle_start_tag, local_name,
    validate_dmn_root_start_tag,
};
use crate::error::{BpmnEngineError, Result};
use quick_xml::Reader;
use quick_xml::events::Event;

struct ParseLoopState {
    decisions: Vec<TempDecision>,
    current_decision: Option<TempDecision>,
    current_literal: Option<TempLiteralExpression>,
    current_list: Option<TempListExpression>,
    current_context: Option<TempContextExpression>,
    current_context_entry: Option<TempContextEntry>,
    current_relation: Option<TempRelationExpression>,
    current_relation_row: Option<TempRelationRow>,
    current_invocation: Option<TempInvocation>,
    current_invocation_binding: Option<TempInvocationBinding>,
    current_table: Option<TempTable>,
    current_input: Option<TempInput>,
    current_output: Option<TempOutput>,
    current_rule: Option<TempRule>,
    capture_target: Option<CaptureTarget>,
    capture_buffer: String,
    element_stack: Vec<String>,
}

impl ParseLoopState {
    fn new() -> Self {
        Self {
            decisions: Vec::new(),
            current_decision: None,
            current_literal: None,
            current_list: None,
            current_context: None,
            current_context_entry: None,
            current_relation: None,
            current_relation_row: None,
            current_invocation: None,
            current_invocation_binding: None,
            current_table: None,
            current_input: None,
            current_output: None,
            current_rule: None,
            capture_target: None,
            capture_buffer: String::new(),
            element_stack: Vec::new(),
        }
    }
}

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

fn handle_parse_event(
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
    event: &quick_xml::events::BytesStart<'_>,
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
            source_id: source.source_id.clone(),
        });
    }
    Ok(())
}

fn finalize_loop_state(
    source: &DmnSourceFile,
    mut state: ParseLoopState,
) -> Result<Vec<DmnDecisionDefinition>> {
    if let Some(decision) = state.current_decision.take() {
        state.decisions.push(decision);
    }

    finalize_decision_definitions(source, state.decisions)
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
