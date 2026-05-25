use super::state::{ensure_native_io, native_io_mut};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::Result;
use crate::parser::import::capture::last_process_node_mut;
use crate::parser::import::model::{RawHumanTaskFormSpec, RawProcess};

pub(in crate::parser::import) fn apply_documentation_text(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    let io = ensure_native_io(source, process)?;
    match &mut io.documentation_text {
        Some(existing) if !existing.is_empty() => {
            existing.push(' ');
            existing.push_str(text);
        }
        _ => io.documentation_text = Some(text.to_string()),
    }
    sync_form_from_native_io(source, process)
}

pub(super) fn sync_form_from_native_io(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
) -> Result<()> {
    let io = native_io_mut(source, process)?.clone();
    let Some(interaction_type) = io.interaction_type else {
        return Ok(());
    };
    let question_text = if io.question_ref.is_some() {
        io.question_text
    } else {
        io.question_text.or(io.documentation_text)
    };
    last_process_node_mut(source, process)?.human_task_form = Some(RawHumanTaskFormSpec {
        interaction_type,
        question_ref: io.question_ref,
        question_text,
        choices_ref: io.choices_ref,
        choices: io.choices,
        free_text_fields: io.free_text_fields,
        result_output: io.result_output,
    });
    Ok(())
}
