use super::contract_shape::issue_from_dmn_contract_error;
use super::contract_subset::issue_from_dmn_table_error;
use super::document_dispatch::issue_from_dmn_document_error;
use super::unexpected::unexpected_dmn_issue;
use crate::dmn_model_api::{DmnDocumentSnapshot, DmnSourceFile};
use crate::dmn_parse_api::parse_dmn_decisions;
use crate::dmn_snapshot_api::snapshot_dmn_source;
use crate::error::BpmnEngineError;
use crate::lint_api::{LintDomain, LintIssue, LintReport};

/// Lints one DMN source and returns an LLM-friendly blocking report.
#[must_use]
pub(crate) fn lint_dmn_source_impl(source: &DmnSourceFile) -> LintReport {
    let snapshot = snapshot_dmn_source(source).ok();
    match parse_dmn_decisions(source) {
        Ok(_) => LintReport::ok(LintDomain::Dmn, &source.source_id),
        Err(error) => LintReport::blocking(
            LintDomain::Dmn,
            &source.source_id,
            vec![issue_from_dmn_error(source, &error, snapshot.as_ref())],
        ),
    }
}

fn issue_from_dmn_error(
    source: &DmnSourceFile,
    error: &BpmnEngineError,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    issue_from_dmn_document_error(error, snapshot)
        .or_else(|| issue_from_dmn_contract_error(error, snapshot))
        .or_else(|| issue_from_dmn_table_error(error, snapshot))
        .unwrap_or_else(|| unexpected_dmn_issue(source, error))
}
