use super::shell::BpmnInstanceState;
use crate::error::{BpmnEngineError, Result};
use crate::ir::{BpmnPackage, BpmnProcessSpec};

pub(crate) fn resolve_process_for_instance<'a>(
    package: &'a BpmnPackage,
    instance: &mut BpmnInstanceState,
) -> Result<&'a BpmnProcessSpec> {
    if let Some(process) = package
        .processes
        .get(instance.process_index as usize)
        .filter(|process| process.key.process_id == instance.process.process_id)
    {
        return Ok(process);
    }

    let (process_index, process) = package
        .find_process_position(instance.process.process_id.as_ref())
        .ok_or_else(|| BpmnEngineError::MissingProcess {
            process_id: instance.process.process_id.to_string(),
        })?;
    instance.process_index = process_index;
    Ok(process)
}
