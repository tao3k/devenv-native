use super::binding::start_association;
use super::binding::{
    apply_assignment_from, apply_assignment_to, apply_source_ref, apply_target_ref,
};
use super::declaration::record_declaration;
use super::form::apply_documentation_text;
use super::state::{is_human_task, is_supported_task, last_node_is_human_task};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::Result;
use crate::parser::import::model::{
    CaptureTarget, RawHumanTaskIoAssociationKind, RawHumanTaskIoDeclarationKind, RawProcess,
};
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[allow(clippy::too_many_arguments)]
pub(in crate::parser::import) fn handle_human_task_io_child_start(
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
    HumanTaskIoStartContext {
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

struct HumanTaskIoStartContext<'a, 'reader, 'event> {
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

impl HumanTaskIoStartContext<'_, '_, '_> {
    fn handle(&mut self) -> Result<bool> {
        if self.handle_documentation_start()? || self.handle_io_container_start()? {
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

    fn handle_documentation_start(&mut self) -> Result<bool> {
        if !is_human_task(self.parent) || self.tag != "documentation" {
            return Ok(false);
        }
        self.capture_text_start(
            CaptureTarget::HumanTaskDocumentationText,
            apply_documentation_text,
        )
    }

    fn handle_io_container_start(&mut self) -> Result<bool> {
        if !is_supported_task(self.parent)
            || !matches!(
                self.tag,
                "ioSpecification" | "dataInputAssociation" | "dataOutputAssociation"
            )
        {
            return Ok(false);
        }
        if is_human_task(self.parent) {
            match self.tag {
                "dataInputAssociation" => {
                    start_association(
                        self.source,
                        self.process,
                        RawHumanTaskIoAssociationKind::DataInput,
                        self.is_empty,
                    )?;
                }
                "dataOutputAssociation" => {
                    start_association(
                        self.source,
                        self.process,
                        RawHumanTaskIoAssociationKind::DataOutput,
                        self.is_empty,
                    )?;
                }
                _ => {}
            }
        }
        Ok(true)
    }

    fn handle_io_specification_child_start(&mut self) -> Result<bool> {
        if !last_node_is_human_task(self.process) {
            return Ok(matches!(
                self.tag,
                "dataInput" | "dataOutput" | "inputSet" | "outputSet"
            ));
        }
        match self.tag {
            "dataInput" => self.record_declaration(RawHumanTaskIoDeclarationKind::DataInput),
            "dataOutput" => self.record_declaration(RawHumanTaskIoDeclarationKind::DataOutput),
            "inputSet" | "outputSet" => Ok(true),
            _ => Ok(false),
        }
    }

    fn record_declaration(&mut self, kind: RawHumanTaskIoDeclarationKind) -> Result<bool> {
        record_declaration(self.source, self.reader, self.event, self.process, kind)?;
        Ok(true)
    }

    fn handle_association_child_start(&mut self) -> Result<bool> {
        if !last_node_is_human_task(self.process) {
            return Ok(matches!(
                self.tag,
                "sourceRef" | "targetRef" | "assignment" | "transformation"
            ));
        }
        match self.tag {
            "sourceRef" => {
                self.capture_text_start(CaptureTarget::HumanTaskIoSourceRef, apply_source_ref)
            }
            "targetRef" => {
                self.capture_text_start(CaptureTarget::HumanTaskIoTargetRef, apply_target_ref)
            }
            "assignment" | "transformation" => Ok(true),
            _ => Ok(false),
        }
    }

    fn handle_assignment_child_start(&mut self) -> Result<bool> {
        if !last_node_is_human_task(self.process) {
            return Ok(matches!(self.tag, "from" | "to"));
        }
        match self.tag {
            "from" => self.capture_text_start(
                CaptureTarget::HumanTaskIoAssignmentFrom,
                apply_assignment_from,
            ),
            "to" => {
                self.capture_text_start(CaptureTarget::HumanTaskIoAssignmentTo, apply_assignment_to)
            }
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
