//! Test-facing bridge for semantic check helpers.

use crate::link_graph::PageIndexNode;
pub use crate::parsers::semantic_check::HashReference;
pub use crate::zhenfa_router::native::audit::SourceFile;

pub use super::types::{
    CheckType, FileAuditReport, FuzzySuggestionData, IssueLocation, NodeStatus,
    SemanticCheckResult, SemanticIssue, WendaoSemanticCheckArgs,
};
pub use super::{EpistemeLoadReport, EpistemePolicyQueryReport};
pub use super::{run_audit_core, wendao_semantic_check};

#[must_use]
pub fn extract_id_references(text: &str) -> Vec<String> {
    crate::parsers::semantic_check::extract_id_references(text)
}

#[must_use]
pub fn extract_hash_references(text: &str) -> Vec<HashReference> {
    crate::parsers::semantic_check::extract_hash_references(text)
}

#[must_use]
pub fn validate_contract(contract: &str, content: &str) -> Option<String> {
    crate::parsers::semantic_check::validate_contract(contract, content)
}

#[must_use]
pub fn extract_function_args<'a>(contract: &'a str, function_name: &str) -> Option<&'a str> {
    crate::parsers::semantic_check::extract_function_args(contract, function_name)
}

#[must_use]
pub fn generate_suggested_id(title: &str) -> String {
    crate::parsers::semantic_check::generate_suggested_id(title)
}

#[must_use]
pub fn xml_escape(s: &str) -> String {
    super::report::xml_escape(s)
}

#[must_use]
pub fn issue_type_to_code(issue_type: &str) -> &'static str {
    super::report::issue_type_to_code(issue_type)
}

#[must_use]
pub fn build_file_reports(issues: &[SemanticIssue], docs: &[String]) -> Vec<FileAuditReport> {
    super::report::build_file_reports(issues, docs)
}

#[must_use]
pub fn format_result_as_xml(result: &SemanticCheckResult) -> String {
    super::report::format_result_as_xml(result)
}

pub fn check_code_observations(
    node: &PageIndexNode,
    doc_id: &str,
    source_files: &[SourceFile],
    fuzzy_threshold: Option<f32>,
    issues: &mut Vec<SemanticIssue>,
) {
    super::checks::check_code_observations(node, doc_id, source_files, fuzzy_threshold, issues);
}
