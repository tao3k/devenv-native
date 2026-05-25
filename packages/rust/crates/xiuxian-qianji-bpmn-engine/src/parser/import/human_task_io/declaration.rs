use super::state::{ensure_native_io, last_node_is_human_task};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::Result;
use crate::parser::import::attributes::attribute_value;
use crate::parser::import::model::{
    RawHumanTaskIoDeclaration, RawHumanTaskIoDeclarationKind, RawProcess,
};
use quick_xml::Reader;
use quick_xml::events::BytesStart;

pub(super) fn record_declaration(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    process: &mut RawProcess,
    kind: RawHumanTaskIoDeclarationKind,
) -> Result<()> {
    if !last_node_is_human_task(process) {
        return Ok(());
    }
    let Some(id) = attribute_value(reader, event, "id")? else {
        return Ok(());
    };
    let Some(name) = attribute_value(reader, event, "name")? else {
        return Ok(());
    };
    let io = ensure_native_io(source, process)?;
    io.declarations
        .push(RawHumanTaskIoDeclaration { id, name, kind });
    Ok(())
}
