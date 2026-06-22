use super::form::sync_form_from_native_io;
use super::literal::{parse_choice_literal, parse_free_text_literal};
use super::state::{
    active_association_mut, ensure_native_io, last_node_is_human_task, native_io_mut,
};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::Result;
use crate::parser::import::model::{
    RawHumanTaskIoAssociation, RawHumanTaskIoAssociationKind, RawHumanTaskIoDeclarationKind,
    RawProcess,
};

pub(in crate::parser::import) fn apply_source_ref(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    let association = active_association_mut(source, process)?;
    association.source_refs.push(text.to_string());
    Ok(())
}

pub(in crate::parser::import) fn apply_target_ref(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    let association = active_association_mut(source, process)?;
    association.target_ref = Some(text.to_string());
    Ok(())
}

pub(in crate::parser::import) fn apply_assignment_from(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    let association = active_association_mut(source, process)?;
    association.assignment_from = Some(text.to_string());
    Ok(())
}

pub(in crate::parser::import) fn apply_assignment_to(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    let association = active_association_mut(source, process)?;
    association.assignment_to = Some(text.to_string());
    Ok(())
}

pub(in crate::parser::import) fn complete_end_tag(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    tag: &str,
) -> Result<()> {
    let expected_kind = match tag {
        "dataInputAssociation" => RawHumanTaskIoAssociationKind::DataInput,
        "dataOutputAssociation" => RawHumanTaskIoAssociationKind::DataOutput,
        _ => return Ok(()),
    };
    if !last_node_is_human_task(process) {
        return Ok(());
    }
    let Some(association) = native_io_mut(source, process)?.active_association.take() else {
        return Ok(());
    };
    if association.kind != expected_kind {
        return Ok(());
    }
    match association.kind {
        RawHumanTaskIoAssociationKind::DataInput => {
            apply_input_association(source, process, &association)?;
        }
        RawHumanTaskIoAssociationKind::DataOutput => {
            apply_output_association(source, process, association)?;
        }
    }
    sync_form_from_native_io(source, process)
}

pub(super) fn start_association(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    kind: RawHumanTaskIoAssociationKind,
    is_empty: bool,
) -> Result<()> {
    let io = ensure_native_io(source, process)?;
    io.active_association = Some(RawHumanTaskIoAssociation::new(kind));
    if is_empty {
        io.active_association = None;
    }
    Ok(())
}

fn apply_input_association(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    association: &RawHumanTaskIoAssociation,
) -> Result<()> {
    let Some(target_ref) = association
        .target_ref
        .as_deref()
        .or(association.assignment_to.as_deref())
    else {
        return Ok(());
    };
    let Some(name) = input_name_for_ref(source, process, target_ref)? else {
        return Ok(());
    };
    let source_ref = association.source_refs.first().map(String::as_str);
    let literal = association.assignment_from.as_deref();
    let io = native_io_mut(source, process)?;
    match name.as_str() {
        "interactionType" => {
            if let Some(value) = literal.map(str::trim).filter(|value| !value.is_empty()) {
                io.interaction_type = Some(value.to_string());
            }
        }
        "question" => {
            if let Some(source_ref) = source_ref {
                io.question_ref = Some(source_ref.to_string());
                io.question_text = None;
            } else if let Some(value) = literal.map(str::trim).filter(|value| !value.is_empty()) {
                io.question_text = Some(value.to_string());
                io.question_ref = None;
            }
        }
        "choices" => {
            if let Some(source_ref) = source_ref {
                io.choices_ref = Some(source_ref.to_string());
            } else if let Some(value) = literal {
                io.choices = parse_choice_literal(value)?;
            }
        }
        "freeText" => {
            if let Some(value) = literal {
                io.free_text_fields = parse_free_text_literal(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn apply_output_association(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    association: RawHumanTaskIoAssociation,
) -> Result<()> {
    let Some(target_ref) = association.target_ref.or(association.assignment_to) else {
        return Ok(());
    };
    let output_name = match association
        .source_refs
        .first()
        .map(String::as_str)
        .or(association.assignment_from.as_deref())
    {
        Some(source_ref) => output_name_for_ref(source, process, source_ref)?,
        None => None,
    };
    if output_name.as_deref() == Some("answer") || output_name.is_none() {
        native_io_mut(source, process)?.result_output = Some(target_ref);
    }
    Ok(())
}

fn input_name_for_ref(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    target_ref: &str,
) -> Result<Option<String>> {
    Ok(native_io_mut(source, process)?
        .declarations
        .iter()
        .find(|declaration| {
            declaration.kind == RawHumanTaskIoDeclarationKind::DataInput
                && declaration.id == target_ref
        })
        .map(|declaration| declaration.name.clone()))
}

fn output_name_for_ref(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    source_ref: &str,
) -> Result<Option<String>> {
    Ok(native_io_mut(source, process)?
        .declarations
        .iter()
        .find(|declaration| {
            declaration.kind == RawHumanTaskIoDeclarationKind::DataOutput
                && declaration.id == source_ref
        })
        .map(|declaration| declaration.name.clone()))
}
