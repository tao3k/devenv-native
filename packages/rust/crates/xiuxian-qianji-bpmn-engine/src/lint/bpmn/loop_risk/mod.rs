//! Canonical api seam for BPMN loop-risk lint checks.

use crate::BpmnPackage;
use crate::BpmnProcessSpec;
use crate::BpmnSourceFile;
use crate::repeat_condition::{GatewayConditionSummary, parse_gateway_condition_summary};
use crate::{BpmnGatewayKind, BpmnNodeKind};
use crate::{LintIssue, LintSourceDiagnostic, LintSourceSpan};
use quick_xml::Reader;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesStart, Event};
use serde_json::{Value, json};
use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::ops::Range;

mod api;
mod fix;
mod graph;
mod issue;
mod metadata;
mod model;
mod variable;
mod xml;

use fix::{
    line_fix_xml_strings, loop_progress_contract_message, loop_progress_help,
    loop_progress_line_fixes, primary_cycle_span,
};
use graph::{
    component_has_exit_path, default_reentry_flows, is_cyclic_component,
    strongly_connected_components,
};
use issue::process_loop_risk_issues;
use metadata::collect_process_metadata;
use model::{
    ActiveTask, DefaultReentryFlow, LoopRiskEvidence, ProcessMetadata, SequenceFlowMetadata,
    TaskAssociationCapture, TaskAssociationContext,
};
use variable::{
    gateway_node_ids, is_host_task, is_prompt_output, is_state_worker_task, route_variables,
    sorted_node_ids, sorted_set_values, task_node_ids, undeclared_variables, updated_variables,
    user_task_outputs, worker_task_inputs, worker_task_outputs,
};
use xml::{
    append_entity_reference, attribute_value, is_span_only_node_tag, is_task_tag, local_name,
    outgoing_edge_indices, parse_variable_names, reader_position, record_node_span,
    start_or_empty_event_span,
};

pub(super) use api::loop_risk_issues;
