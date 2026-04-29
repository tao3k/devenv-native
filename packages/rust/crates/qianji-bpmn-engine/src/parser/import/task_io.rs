use super::attributes::attribute_value;
use super::capture::last_process_node_mut;
use super::model::{
    CaptureTarget, RawProcess, RawTaskInputBinding, RawTaskInputSource, RawTaskIoAssociation,
    RawTaskIoAssociationKind, RawTaskIoDeclaration, RawTaskIoDeclarationKind, RawTaskIoSpec,
    RawTaskOutputBinding,
};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use crate::ir_node_api::BpmnNodeKind;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_task_io_child_start(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    parent: &str,
    process: &mut RawProcess,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
    is_empty: bool,
) -> Result<bool> {
    TaskIoStartContext {
        source,
        reader,
        event,
        tag,
        parent,
        process,
        capture_target,
        capture_buffer,
        is_empty,
    }
    .handle()
}

struct TaskIoStartContext<'a, 'reader, 'event> {
    source: &'a BpmnSourceFile,
    reader: &'a Reader<&'reader [u8]>,
    event: &'a BytesStart<'event>,
    tag: &'a str,
    parent: &'a str,
    process: &'a mut RawProcess,
    capture_target: &'a mut Option<CaptureTarget>,
    capture_buffer: &'a mut String,
    is_empty: bool,
}

impl TaskIoStartContext<'_, '_, '_> {
    fn handle(&mut self) -> Result<bool> {
        if self.handle_io_container_start()? {
            return Ok(true);
        }
        match self.parent {
            "ioSpecification" => self.handle_io_specification_child_start(),
            "inputSet" | "outputSet" => Ok(matches!(self.tag, "dataInputRefs" | "dataOutputRefs")),
            "dataInputAssociation" | "dataOutputAssociation" => {
                self.handle_association_child_start()
            }
            "assignment" => self.handle_assignment_child_start(),
            _ => Ok(false),
        }
    }

    fn handle_io_container_start(&mut self) -> Result<bool> {
        if !last_node_is_task_io_owner(self.process)
            || !matches!(
                self.tag,
                "ioSpecification" | "dataInputAssociation" | "dataOutputAssociation"
            )
        {
            return Ok(false);
        }
        match self.tag {
            "dataInputAssociation" => {
                start_association(
                    self.source,
                    self.process,
                    RawTaskIoAssociationKind::DataInput,
                    self.is_empty,
                )?;
            }
            "dataOutputAssociation" => {
                start_association(
                    self.source,
                    self.process,
                    RawTaskIoAssociationKind::DataOutput,
                    self.is_empty,
                )?;
            }
            _ => {}
        }
        Ok(true)
    }

    fn handle_io_specification_child_start(&mut self) -> Result<bool> {
        if !last_node_is_task_io_owner(self.process) {
            return Ok(false);
        }
        match self.tag {
            "dataInput" => self.record_declaration(RawTaskIoDeclarationKind::DataInput),
            "dataOutput" => self.record_declaration(RawTaskIoDeclarationKind::DataOutput),
            "inputSet" | "outputSet" => Ok(true),
            _ => Ok(false),
        }
    }

    fn record_declaration(&mut self, kind: RawTaskIoDeclarationKind) -> Result<bool> {
        record_declaration(self.source, self.reader, self.event, self.process, kind)?;
        Ok(true)
    }

    fn handle_association_child_start(&mut self) -> Result<bool> {
        if !last_node_is_task_io_owner(self.process) {
            return Ok(false);
        }
        match self.tag {
            "sourceRef" => {
                self.capture_text_start(CaptureTarget::TaskIoSourceRef, apply_task_io_source_ref)
            }
            "targetRef" => {
                self.capture_text_start(CaptureTarget::TaskIoTargetRef, apply_task_io_target_ref)
            }
            "assignment" => Ok(true),
            "transformation" => {
                let process_id = self.process.process_id.clone();
                let node_id = last_process_node_mut(self.source, self.process)?
                    .bpmn_id
                    .clone();
                Err(BpmnEngineError::UnsupportedTaskConfiguration {
                    process_id,
                    node_id,
                    detail: "task_io_transformation_deferred",
                })
            }
            _ => Ok(false),
        }
    }

    fn handle_assignment_child_start(&mut self) -> Result<bool> {
        if !last_node_is_task_io_owner(self.process) {
            return Ok(false);
        }
        match self.tag {
            "from" => self.capture_text_start(
                CaptureTarget::TaskIoAssignmentFrom,
                apply_task_io_assignment_from,
            ),
            "to" => self.capture_text_start(
                CaptureTarget::TaskIoAssignmentTo,
                apply_task_io_assignment_to,
            ),
            _ => Ok(false),
        }
    }

    fn capture_text_start(
        &mut self,
        target: CaptureTarget,
        apply_empty: fn(&BpmnSourceFile, &mut RawProcess, &str) -> Result<()>,
    ) -> Result<bool> {
        *self.capture_target = Some(target);
        self.capture_buffer.clear();
        if self.is_empty {
            apply_empty(self.source, self.process, "")?;
            *self.capture_target = None;
            self.capture_buffer.clear();
        }
        Ok(true)
    }
}

pub(super) fn complete_task_io_end_tag(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    tag: &str,
) -> Result<()> {
    let expected_kind = match tag {
        "dataInputAssociation" => RawTaskIoAssociationKind::DataInput,
        "dataOutputAssociation" => RawTaskIoAssociationKind::DataOutput,
        _ => return Ok(()),
    };
    if !last_node_is_task_io_owner(process) {
        return Ok(());
    }
    let Some(association) = task_io_mut(source, process)?.active_association.take() else {
        return Ok(());
    };
    if association.kind != expected_kind {
        return Ok(());
    }
    match association.kind {
        RawTaskIoAssociationKind::DataInput => {
            apply_input_association(source, process, &association)
        }
        RawTaskIoAssociationKind::DataOutput => {
            apply_output_association(source, process, &association)
        }
    }
}

pub(super) fn apply_task_io_source_ref(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    active_association_mut(source, process)?
        .source_refs
        .push(text.to_string());
    Ok(())
}

pub(super) fn apply_task_io_target_ref(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    active_association_mut(source, process)?.target_ref = Some(text.to_string());
    Ok(())
}

pub(super) fn apply_task_io_assignment_from(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    active_association_mut(source, process)?.assignment_from = Some(text.to_string());
    Ok(())
}

pub(super) fn apply_task_io_assignment_to(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    active_association_mut(source, process)?.assignment_to = Some(text.to_string());
    Ok(())
}

fn record_declaration(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    process: &mut RawProcess,
    kind: RawTaskIoDeclarationKind,
) -> Result<()> {
    let Some(id) = attribute_value(reader, event, "id")? else {
        return Ok(());
    };
    let Some(name) = attribute_value(reader, event, "name")? else {
        return Ok(());
    };
    task_io_mut(source, process)?
        .declarations
        .push(RawTaskIoDeclaration { id, name, kind });
    Ok(())
}

fn start_association(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    kind: RawTaskIoAssociationKind,
    is_empty: bool,
) -> Result<()> {
    let io = task_io_mut(source, process)?;
    io.active_association = Some(RawTaskIoAssociation::new(kind));
    if is_empty {
        io.active_association = None;
    }
    Ok(())
}

fn apply_input_association(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    association: &RawTaskIoAssociation,
) -> Result<()> {
    ensure_single_source_ref(source, process, association)?;
    let Some(target_ref) = association
        .target_ref
        .as_deref()
        .or(association.assignment_to.as_deref())
    else {
        return missing_task_io_binding(source, process, "task_io_input_missing_target");
    };
    let input_name = declaration_name_for_ref(
        source,
        process,
        target_ref,
        RawTaskIoDeclarationKind::DataInput,
        "task_io_input_target_not_declared",
    )?;
    let binding_source = match (
        association.source_refs.first(),
        association.assignment_from.as_deref(),
    ) {
        (Some(source_ref), None) => RawTaskInputSource::Variable {
            source_ref: source_ref.clone(),
        },
        (None, Some(value)) if !value.trim().is_empty() => RawTaskInputSource::Literal {
            value: value.trim().to_string(),
        },
        _ => return missing_task_io_binding(source, process, "task_io_input_missing_source"),
    };
    task_io_mut(source, process)?
        .inputs
        .push(RawTaskInputBinding {
            name: input_name,
            source: binding_source,
        });
    Ok(())
}

fn apply_output_association(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    association: &RawTaskIoAssociation,
) -> Result<()> {
    ensure_single_source_ref(source, process, association)?;
    let Some(output_ref) = association
        .source_refs
        .first()
        .map(String::as_str)
        .or(association.assignment_from.as_deref())
    else {
        return missing_task_io_binding(source, process, "task_io_output_missing_source");
    };
    let output_name = declaration_name_for_ref(
        source,
        process,
        output_ref,
        RawTaskIoDeclarationKind::DataOutput,
        "task_io_output_source_not_declared",
    )?;
    let Some(target_ref) = association
        .target_ref
        .clone()
        .or_else(|| association.assignment_to.clone())
    else {
        return missing_task_io_binding(source, process, "task_io_output_missing_target");
    };
    task_io_mut(source, process)?
        .outputs
        .push(RawTaskOutputBinding {
            name: output_name,
            target_ref,
        });
    Ok(())
}

fn ensure_single_source_ref(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    association: &RawTaskIoAssociation,
) -> Result<()> {
    if association.source_refs.len() > 1 {
        let process_id = process.process_id.clone();
        let node_id = last_process_node_mut(source, process)?.bpmn_id.clone();
        return Err(BpmnEngineError::UnsupportedTaskConfiguration {
            process_id,
            node_id,
            detail: "task_io_multiple_source_refs_deferred",
        });
    }
    Ok(())
}

fn declaration_name_for_ref(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    reference: &str,
    kind: RawTaskIoDeclarationKind,
    detail: &'static str,
) -> Result<String> {
    if let Some(name) = task_io_mut(source, process)?
        .declarations
        .iter()
        .find(|declaration| declaration.kind == kind && declaration.id == reference)
        .map(|declaration| declaration.name.clone())
    {
        return Ok(name);
    }
    let process_id = process.process_id.clone();
    let node_id = last_process_node_mut(source, process)?.bpmn_id.clone();
    Err(BpmnEngineError::UnsupportedTaskConfiguration {
        process_id,
        node_id,
        detail,
    })
}

fn missing_task_io_binding<T>(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    detail: &'static str,
) -> Result<T> {
    let process_id = process.process_id.clone();
    let node_id = last_process_node_mut(source, process)?.bpmn_id.clone();
    Err(BpmnEngineError::UnsupportedTaskConfiguration {
        process_id,
        node_id,
        detail,
    })
}

fn active_association_mut<'a>(
    source: &BpmnSourceFile,
    process: &'a mut RawProcess,
) -> Result<&'a mut RawTaskIoAssociation> {
    task_io_mut(source, process)?
        .active_association
        .as_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "task_io_association_child_without_association",
        })
}

fn task_io_mut<'a>(
    source: &BpmnSourceFile,
    process: &'a mut RawProcess,
) -> Result<&'a mut RawTaskIoSpec> {
    let node = last_process_node_mut(source, process)?;
    if !is_task_io_owner(&node.kind) {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "task_io_without_supported_task",
        });
    }
    if node.task_io.is_none() {
        node.task_io = Some(RawTaskIoSpec::default());
    }
    node.task_io
        .as_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "task_io_missing_state",
        })
}

fn last_node_is_task_io_owner(process: &RawProcess) -> bool {
    process
        .nodes
        .last()
        .is_some_and(|node| is_task_io_owner(&node.kind))
}

fn is_task_io_owner(kind: &BpmnNodeKind) -> bool {
    matches!(
        kind,
        BpmnNodeKind::SendTask
            | BpmnNodeKind::ReceiveTask
            | BpmnNodeKind::ServiceTask
            | BpmnNodeKind::ScriptTask
            | BpmnNodeKind::BusinessRuleTask
    )
}
