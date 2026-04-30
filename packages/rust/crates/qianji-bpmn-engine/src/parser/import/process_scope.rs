use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use crate::parser::import::attributes::{attribute_value, required_attribute};
use crate::parser::import::{NestedShellKind, RawProcess, RawProcessScope};
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[allow(clippy::too_many_arguments)]
pub(in crate::parser::import) fn handle_package_start_tag(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    package_id: &mut Option<String>,
    process_stack: &mut Vec<RawProcess>,
    processes: &mut Vec<RawProcess>,
    is_empty: bool,
) -> Result<bool> {
    if tag == "definitions" {
        if package_id.is_none() {
            *package_id = attribute_value(reader, event, "id")?;
        }
        return Ok(true);
    }
    if !process_stack.is_empty() || tag != "process" {
        return Ok(false);
    }

    let process_id = required_attribute(source, reader, event, "process", "id")?;
    process_stack.push(RawProcess::new_top_level(process_id));
    if is_empty {
        complete_process_scope("process", process_stack, processes);
    }
    Ok(true)
}

pub(in crate::parser::import) fn complete_process_scope(
    tag: &str,
    process_stack: &mut Vec<RawProcess>,
    processes: &mut Vec<RawProcess>,
) {
    let should_pop = matches!(
        (tag, process_stack.last().map(|process| &process.scope)),
        ("process", Some(RawProcessScope::TopLevel))
            | (
                "subProcess",
                Some(
                    RawProcessScope::NestedShell {
                        kind: NestedShellKind::EmbeddedSubProcess,
                        ..
                    } | RawProcessScope::NestedShell {
                        kind: NestedShellKind::EventSubProcess,
                        ..
                    }
                )
            )
            | (
                "transaction",
                Some(RawProcessScope::NestedShell {
                    kind: NestedShellKind::Transaction,
                    ..
                })
            )
    );
    if should_pop && let Some(process) = process_stack.pop() {
        processes.push(process);
    }
}

pub(in crate::parser::import) fn current_process_mut<'a>(
    process_stack: &'a mut [RawProcess],
    operation: &'static str,
) -> Result<&'a mut RawProcess> {
    process_stack
        .last_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation { operation })
}

pub(in crate::parser::import) fn is_process_scope_tag(tag: &str) -> bool {
    matches!(tag, "process" | "subProcess" | "transaction")
}
