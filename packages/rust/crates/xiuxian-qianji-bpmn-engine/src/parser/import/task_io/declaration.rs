use super::state::task_io_mut;
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::Result;
use crate::parser::import::attributes::attribute_value;
use crate::parser::import::model::{RawProcess, RawTaskIoDeclaration, RawTaskIoDeclarationKind};
use quick_xml::Reader;
use quick_xml::events::BytesStart;

pub(super) fn record_declaration(
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
