use super::binding::TaskIoBindings;
use super::declaration::record_declaration;
use super::state::last_node_is_task_io_owner;
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use crate::parser::import::capture::last_process_node_mut;
use crate::parser::import::model::{
    CaptureTarget, RawProcess, RawTaskIoAssociationKind, RawTaskIoDeclarationKind,
};
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[allow(clippy::too_many_arguments)]
pub(in crate::parser::import) fn handle_task_io_child_start(
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
                TaskIoBindings::start_association(
                    self.source,
                    self.process,
                    RawTaskIoAssociationKind::DataInput,
                    self.is_empty,
                )?;
            }
            "dataOutputAssociation" => {
                TaskIoBindings::start_association(
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
            "sourceRef" => self.capture_text_start(
                CaptureTarget::TaskIoSourceRef,
                TaskIoBindings::apply_source_ref,
            ),
            "targetRef" => self.capture_text_start(
                CaptureTarget::TaskIoTargetRef,
                TaskIoBindings::apply_target_ref,
            ),
            "assignment" => Ok(true),
            "transformation" => {
                let process_id = self.process.process_id.clone();
                let node_id = last_process_node_mut(self.source, self.process)?
                    .bpmn_id
                    .clone();
                Err(BpmnEngineError::UnsupportedTaskConfiguration {
                    process_id: process_id.into(),
                    node_id: node_id.into(),
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
                TaskIoBindings::apply_assignment_from,
            ),
            "to" => self.capture_text_start(
                CaptureTarget::TaskIoAssignmentTo,
                TaskIoBindings::apply_assignment_to,
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
