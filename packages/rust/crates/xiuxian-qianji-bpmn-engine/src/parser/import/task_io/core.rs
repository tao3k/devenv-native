//! Parser import task I/O core operations.

use crate::parser::import::RawProcess;
use crate::{BpmnEngineError, BpmnSourceFile};
type Result<T> = std::result::Result<T, BpmnEngineError>;

use super::binding::TaskIoBindings;

pub(in crate::parser::import) fn complete_task_io_end_tag(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    tag: &str,
) -> Result<()> {
    TaskIoBindings::complete_end_tag(source, process, tag)
}

pub(in crate::parser::import) fn apply_task_io_source_ref(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    TaskIoBindings::apply_source_ref(source, process, text)
}

pub(in crate::parser::import) fn apply_task_io_target_ref(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    TaskIoBindings::apply_target_ref(source, process, text)
}

pub(in crate::parser::import) fn apply_task_io_assignment_from(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    TaskIoBindings::apply_assignment_from(source, process, text)
}

pub(in crate::parser::import) fn apply_task_io_assignment_to(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    TaskIoBindings::apply_assignment_to(source, process, text)
}
