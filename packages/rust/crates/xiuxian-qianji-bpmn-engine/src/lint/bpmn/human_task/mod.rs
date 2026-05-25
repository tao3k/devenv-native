//! Canonical api seam for BPMN human-task lint checks.

use crate::BpmnEngineError;
use crate::BpmnSourceFile;
use crate::{LintIssue, LintSourceDiagnostic, LintSourceSpan};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use serde_json::json;
use std::borrow::Cow;
use std::ops::Range;

mod api;
mod issue;
mod model;
mod scan;
mod xml;

use issue::{
    native_rendering_issue, unsupported_assignment_child_issue,
    unsupported_assignment_semantics_issue, unsupported_global_task_binding_issue,
};
use model::{CallActivityContext, GlobalTaskContext, HumanTaskContext, ProcessContext};
use scan::HumanTaskStandardScanState;
use xml::{
    attribute_value, event_span, is_assignment_role, is_global_task, is_human_interaction_task,
    is_unsupported_assignment_role, local_name, source_diagnostic, source_diagnostic_from_span,
};

pub(super) use api::{human_task_standard_issues, issue_from_bpmn_human_task_standard_error};
