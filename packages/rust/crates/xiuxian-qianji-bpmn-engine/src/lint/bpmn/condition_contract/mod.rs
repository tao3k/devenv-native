//! Canonical api seam for BPMN gateway condition lint contracts.

use crate::BpmnPackage;
use crate::BpmnSourceFile;
use crate::repeat_condition::{
    GatewayConditionSummary, is_supported_gateway_condition, parse_gateway_condition_summary,
};
use crate::{LintIssue, LintSourceDiagnostic, LintSourceSpan};
use quick_xml::Reader;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesStart, Event};
use serde_json::{Value, json};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

mod api;
mod guidance;
mod interaction;
mod issue;
mod model;
mod path;
mod source;
mod xml;

use guidance::{
    ambiguous_boolean_condition_guidance, ambiguous_boolean_condition_help,
    ambiguous_boolean_condition_repair,
};
use interaction::collect_static_interaction_choice_outputs;
use issue::{
    ambiguous_boolean_condition_issue, non_boolean_interaction_choice_condition_issue,
    unsupported_gateway_condition_issue,
};
use model::{
    ActiveGatewayFlow, AmbiguousBooleanPathKind, StaticInteractionChoiceOutput,
    UnsupportedGatewayCondition, UnsupportedGatewayConditionGroup,
};
use path::{ambiguous_boolean_path_kind, collect_gateway_ids, is_boolean_interaction_choice_value};
use source::{
    grouped_unsupported_gateway_condition_issues, source_ambiguous_boolean_condition_issue,
    source_unsupported_gateway_condition,
};
use xml::{
    append_entity_reference, attribute_value, find_condition_expression_span, is_element,
    local_name,
};

pub(super) use api::{
    ambiguous_boolean_gateway_condition_issues, ambiguous_boolean_gateway_condition_source_issues,
    unsupported_gateway_condition_source_issues,
};
