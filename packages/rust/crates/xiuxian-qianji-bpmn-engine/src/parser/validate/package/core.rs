//! BPMN validation for the bounded supported subset.

use crate::BpmnEngineError;
use crate::ir_node_api::BpmnNodeKind;
use crate::parser::import::{NestedShellKind, RawPackageDocument, RawProcess, RawProcessScope};
use std::collections::{HashMap, HashSet};

use super::{compensation, routing, topology};
use crate::parser::validate::{error_paths, recursion};

type Result<T> = std::result::Result<T, BpmnEngineError>;

pub(crate) fn validate_raw_package(raw: &RawPackageDocument) -> Result<()> {
    ensure_process_definitions(raw)?;
    let all_process_ids = raw
        .processes
        .iter()
        .map(|process| process.process_id.as_str())
        .collect::<HashSet<_>>();
    let process_by_id = raw
        .processes
        .iter()
        .map(|process| (process.process_id.as_str(), process))
        .collect::<HashMap<_, _>>();
    let call_activity_owners = error_paths::collect_call_activity_owners(raw);
    let mut seen_process_ids = HashSet::new();
    for process in &raw.processes {
        ensure_unique_process_id(raw, process, &mut seen_process_ids)?;
        let node_ids = collect_node_ids(process)?;
        topology::validate_process_topology(
            process,
            &all_process_ids,
            &node_ids,
            &process_by_id,
            &call_activity_owners,
        )?;
        routing::validate_sequence_flows(process, &node_ids)?;
        routing::validate_standard_loops(process)?;
        routing::validate_multi_instances(process)?;
        compensation::validate_compensation_handlers(process)?;
        routing::validate_task_routing(process)?;
        routing::validate_gateways(process)?;
        routing::validate_event_based_gateways(process, &node_ids)?;
    }

    recursion::detect_recursive_call_activity(raw)?;

    Ok(())
}

fn ensure_process_definitions(raw: &RawPackageDocument) -> Result<()> {
    if raw.processes.is_empty() {
        return Err(BpmnEngineError::MissingProcessDefinitions {
            source_id: (raw.source_id.clone()).into(),
        });
    }
    Ok(())
}

fn ensure_unique_process_id<'a>(
    raw: &RawPackageDocument,
    process: &'a RawProcess,
    seen_process_ids: &mut HashSet<&'a str>,
) -> Result<()> {
    if seen_process_ids.insert(process.process_id.as_str()) {
        return Ok(());
    }
    Err(BpmnEngineError::DuplicateProcessId {
        package_id: (raw.package_id.clone()).into(),
        process_id: (process.process_id.clone()).into(),
    })
}

fn collect_node_ids(process: &RawProcess) -> Result<HashSet<&str>> {
    let scan = scan_node_ids(process)?;
    validate_start_event_count(process, scan.start_event_count)?;
    validate_has_end_event(process, scan.start_event_count, scan.has_end_event)?;
    Ok(scan.node_ids)
}

struct NodeIdScan<'a> {
    node_ids: HashSet<&'a str>,
    start_event_count: usize,
    has_end_event: bool,
}

fn scan_node_ids(process: &RawProcess) -> Result<NodeIdScan<'_>> {
    process.nodes.iter().try_fold(
        NodeIdScan {
            node_ids: HashSet::new(),
            start_event_count: 0,
            has_end_event: false,
        },
        |mut scan, node| {
            if !scan.node_ids.insert(node.bpmn_id.as_str()) {
                return Err(BpmnEngineError::DuplicateNodeId {
                    process_id: (process.process_id.clone()).into(),
                    node_id: (node.bpmn_id.clone()).into(),
                });
            }
            if matches!(node.kind, BpmnNodeKind::StartEvent) {
                scan.start_event_count += 1;
            }
            scan.has_end_event |= matches!(node.kind, BpmnNodeKind::EndEvent);
            Ok(scan)
        },
    )
}

fn validate_start_event_count(process: &RawProcess, start_event_count: usize) -> Result<()> {
    match &process.scope {
        RawProcessScope::TopLevel => {
            if start_event_count == 0 {
                return Err(BpmnEngineError::MissingRequiredProcessElement {
                    process_id: (process.process_id.clone()).into(),
                    element: "start_event",
                });
            }
        }
        RawProcessScope::NestedShell {
            owner_process_id,
            owner_node_id,
            kind,
        } => {
            if is_empty_embedded_metadata_process(process, *kind) {
                return Ok(());
            }
            if start_event_count != 1 {
                return Err(BpmnEngineError::UnsupportedSubProcessConfiguration {
                    process_id: (owner_process_id.clone()).into(),
                    node_id: (owner_node_id.clone()).into(),
                    detail: nested_shell_start_event_detail(*kind),
                });
            }
        }
    }
    Ok(())
}

fn validate_has_end_event(
    process: &RawProcess,
    start_event_count: usize,
    has_end_event: bool,
) -> Result<()> {
    if !has_end_event && is_top_level_start_only_process(process, start_event_count) {
        return Ok(());
    }
    if !has_end_event && is_empty_embedded_metadata_shell(process) {
        return Ok(());
    }
    if !has_end_event && matches!(process.scope, RawProcessScope::TopLevel) {
        return Ok(());
    }
    if !has_end_event {
        return Err(match &process.scope {
            RawProcessScope::TopLevel => BpmnEngineError::MissingRequiredProcessElement {
                process_id: (process.process_id.clone()).into(),
                element: "end_event",
            },
            RawProcessScope::NestedShell {
                owner_process_id,
                owner_node_id,
                kind,
            } => BpmnEngineError::UnsupportedSubProcessConfiguration {
                process_id: (owner_process_id.clone()).into(),
                node_id: (owner_node_id.clone()).into(),
                detail: nested_shell_missing_end_detail(*kind),
            },
        });
    }
    Ok(())
}

fn is_top_level_start_only_process(process: &RawProcess, start_event_count: usize) -> bool {
    matches!(process.scope, RawProcessScope::TopLevel)
        && start_event_count == 1
        && process.nodes.len() == 1
        && process.flows.is_empty()
        && process.associations.is_empty()
}

fn is_empty_embedded_metadata_shell(process: &RawProcess) -> bool {
    match process.scope {
        RawProcessScope::NestedShell {
            kind: NestedShellKind::EmbeddedSubProcess,
            ..
        } => {
            process.nodes.is_empty() && process.flows.is_empty() && process.associations.is_empty()
        }
        _ => false,
    }
}

fn is_empty_embedded_metadata_process(process: &RawProcess, kind: NestedShellKind) -> bool {
    kind == NestedShellKind::EmbeddedSubProcess
        && process.nodes.is_empty()
        && process.flows.is_empty()
        && process.associations.is_empty()
}

fn nested_shell_start_event_detail(kind: NestedShellKind) -> &'static str {
    match kind {
        NestedShellKind::EmbeddedSubProcess => "embedded_subprocess_start_event_count",
        NestedShellKind::Transaction => "transaction_start_event_count",
        NestedShellKind::EventSubProcess => "event_subprocess_start_event_count",
    }
}

fn nested_shell_missing_end_detail(kind: NestedShellKind) -> &'static str {
    match kind {
        NestedShellKind::EmbeddedSubProcess => "embedded_subprocess_missing_end_event",
        NestedShellKind::Transaction => "transaction_missing_end_event",
        NestedShellKind::EventSubProcess => "event_subprocess_missing_end_event",
    }
}
