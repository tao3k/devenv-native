use super::model::RawProcess;
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::Result;

mod binding;
mod declaration;
mod start;
mod state;

use binding::TaskIoBindings;
pub(super) use start::handle_task_io_child_start;

pub(super) fn complete_task_io_end_tag(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    tag: &str,
) -> Result<()> {
    TaskIoBindings::complete_end_tag(source, process, tag)
}

pub(super) fn apply_task_io_source_ref(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    TaskIoBindings::apply_source_ref(source, process, text)
}

pub(super) fn apply_task_io_target_ref(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    TaskIoBindings::apply_target_ref(source, process, text)
}

pub(super) fn apply_task_io_assignment_from(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    TaskIoBindings::apply_assignment_from(source, process, text)
}

pub(super) fn apply_task_io_assignment_to(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    TaskIoBindings::apply_assignment_to(source, process, text)
}
