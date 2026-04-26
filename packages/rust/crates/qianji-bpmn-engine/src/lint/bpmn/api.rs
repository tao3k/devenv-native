use super::document::issue_from_bpmn_document_error;
use super::document_surface::deferred_document_surface_issue;
use super::execution::issue_from_bpmn_execution_shape_error;
use super::extension::qianji_extension_issue;
use super::identity::issue_from_bpmn_identity_error;
use super::reference::issue_from_bpmn_reference_error;
use super::topology::issue_from_bpmn_topology_error;
use super::unexpected::unexpected_bpmn_issue;
use crate::bpmn_parse_api::{BpmnParseOptions, BpmnSourceFile, parse_bpmn_package};
use crate::error::BpmnEngineError;
use crate::lint_api::{LintDomain, LintIssue, LintReport};

/// Lints one BPMN source and returns an LLM-friendly blocking report.
#[must_use]
pub(crate) fn lint_bpmn_source_impl(source: &BpmnSourceFile) -> LintReport {
    if let Some(issue) = deferred_document_surface_issue(source) {
        return LintReport::blocking(LintDomain::Bpmn, &source.source_id, vec![issue]);
    }

    match parse_bpmn_package(std::slice::from_ref(source), &BpmnParseOptions::default()) {
        Ok(_) => match qianji_extension_issue(source) {
            Some(issue) => LintReport::blocking(LintDomain::Bpmn, &source.source_id, vec![issue]),
            None => LintReport::ok(LintDomain::Bpmn, &source.source_id),
        },
        Err(error) => LintReport::blocking(
            LintDomain::Bpmn,
            &source.source_id,
            vec![issue_from_bpmn_error(source, &error)],
        ),
    }
}

fn issue_from_bpmn_error(source: &BpmnSourceFile, error: &BpmnEngineError) -> LintIssue {
    issue_from_bpmn_document_error(error)
        .or_else(|| issue_from_bpmn_identity_error(error))
        .or_else(|| issue_from_bpmn_reference_error(error))
        .or_else(|| issue_from_bpmn_topology_error(error))
        .or_else(|| issue_from_bpmn_execution_shape_error(error))
        .unwrap_or_else(|| unexpected_bpmn_issue(source, error))
}
