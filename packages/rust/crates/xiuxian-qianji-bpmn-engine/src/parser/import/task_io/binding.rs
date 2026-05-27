use super::state::task_io_mut;
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use crate::parser::import::capture::last_process_node_mut;
use crate::parser::import::model::{
    RawProcess, RawTaskInputBinding, RawTaskInputSource, RawTaskIoAssociation,
    RawTaskIoAssociationKind, RawTaskIoDeclarationKind, RawTaskOutputBinding,
};

pub(super) struct TaskIoBindings;

impl TaskIoBindings {
    pub(super) fn complete_end_tag(
        source: &BpmnSourceFile,
        process: &mut RawProcess,
        tag: &str,
    ) -> Result<()> {
        let expected_kind = match tag {
            "dataInputAssociation" => RawTaskIoAssociationKind::DataInput,
            "dataOutputAssociation" => RawTaskIoAssociationKind::DataOutput,
            _ => return Ok(()),
        };
        if process
            .nodes
            .last()
            .and_then(|node| node.task_io.as_ref())
            .is_none()
        {
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
                Self::apply_input_association(source, process, &association)
            }
            RawTaskIoAssociationKind::DataOutput => {
                Self::apply_output_association(source, process, &association)
            }
        }
    }

    pub(super) fn apply_source_ref(
        source: &BpmnSourceFile,
        process: &mut RawProcess,
        text: &str,
    ) -> Result<()> {
        let text = text.trim();
        if text.is_empty() {
            return Ok(());
        }
        Self::active_association_mut(source, process)?
            .source_refs
            .push(text.to_string());
        Ok(())
    }

    pub(super) fn apply_target_ref(
        source: &BpmnSourceFile,
        process: &mut RawProcess,
        text: &str,
    ) -> Result<()> {
        let text = text.trim();
        if text.is_empty() {
            return Ok(());
        }
        Self::active_association_mut(source, process)?.target_ref = Some(text.to_string());
        Ok(())
    }

    pub(super) fn apply_assignment_from(
        source: &BpmnSourceFile,
        process: &mut RawProcess,
        text: &str,
    ) -> Result<()> {
        let text = text.trim();
        if text.is_empty() {
            return Ok(());
        }
        Self::active_association_mut(source, process)?.assignment_from = Some(text.to_string());
        Ok(())
    }

    pub(super) fn apply_assignment_to(
        source: &BpmnSourceFile,
        process: &mut RawProcess,
        text: &str,
    ) -> Result<()> {
        let text = text.trim();
        if text.is_empty() {
            return Ok(());
        }
        Self::active_association_mut(source, process)?.assignment_to = Some(text.to_string());
        Ok(())
    }

    pub(super) fn start_association(
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
        Self::ensure_single_source_ref(source, process, association)?;
        let Some(target_ref) = association
            .target_ref
            .as_deref()
            .or(association.assignment_to.as_deref())
        else {
            return Self::missing_task_io_binding(source, process, "task_io_input_missing_target");
        };
        if Self::is_task_property_ref(source, process, target_ref)? {
            return Ok(());
        }
        let input_name = Self::declaration_name_for_ref(
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
            _ => {
                return Self::missing_task_io_binding(
                    source,
                    process,
                    "task_io_input_missing_source",
                );
            }
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
        Self::ensure_single_source_ref(source, process, association)?;
        let Some(output_ref) = association
            .source_refs
            .first()
            .map(String::as_str)
            .or(association.assignment_from.as_deref())
        else {
            return Self::missing_task_io_binding(source, process, "task_io_output_missing_source");
        };
        if Self::is_task_property_ref(source, process, output_ref)? {
            return Ok(());
        }
        let output_name = Self::declaration_name_for_ref(
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
            return Self::missing_task_io_binding(source, process, "task_io_output_missing_target");
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
                process_id: process_id.into(),
                node_id: node_id.into(),
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
            process_id: process_id.into(),
            node_id: node_id.into(),
            detail,
        })
    }

    fn is_task_property_ref(
        source: &BpmnSourceFile,
        process: &mut RawProcess,
        reference: &str,
    ) -> Result<bool> {
        Ok(task_io_mut(source, process)?
            .property_ids
            .iter()
            .any(|property_id| property_id == reference))
    }

    fn missing_task_io_binding<T>(
        source: &BpmnSourceFile,
        process: &mut RawProcess,
        detail: &'static str,
    ) -> Result<T> {
        let process_id = process.process_id.clone();
        let node_id = last_process_node_mut(source, process)?.bpmn_id.clone();
        Err(BpmnEngineError::UnsupportedTaskConfiguration {
            process_id: process_id.into(),
            node_id: node_id.into(),
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
}
