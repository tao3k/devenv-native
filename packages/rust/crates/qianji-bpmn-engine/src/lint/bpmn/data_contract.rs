//! Canonical api seam for BPMN data contract lint checks.

use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint_api::{LintIssue, LintSourceDiagnostic, LintSourceSpan};
use crate::repeat_condition::{GatewayConditionSummary, parse_gateway_condition_summary};
use quick_xml::Reader;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesStart, Event};
use serde_json::json;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::ops::Range;

mod api;
mod collector;
mod condition;
mod issue;
mod model;
mod xml;

use collector::{SequenceFlowContract, collect_process_contracts};
use condition::{declares_gateway_variable, gateway_condition_variable_path, is_task_tag};
use issue::{UndeclaredGatewayConditionIssue, undeclared_gateway_condition_output_issue};
use model::ProcessContract;
use xml::{
    append_entity_reference, attribute_value, local_name, parse_output_names, reader_position,
    start_event_span,
};

pub(super) use api::undeclared_gateway_condition_output_issues;
