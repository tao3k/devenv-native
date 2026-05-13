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
/// `extract_id_references` public function boundary for Wendao.
#[must_use]
pub fn extract_id_references(text: &str) -> Vec<String> {
    crate::parsers::semantic_check::extract_id_references(text)
}
/// `extract_hash_references` public function boundary for Wendao.
#[must_use]
pub fn extract_hash_references(text: &str) -> Vec<HashReference> {
    crate::parsers::semantic_check::extract_hash_references(text)
}
/// `validate_contract` public function boundary for Wendao.
#[must_use]
pub fn validate_contract(contract: &str, content: &str) -> Option<String> {
    crate::parsers::semantic_check::validate_contract(contract, content)
}
/// `extract_function_args` public function boundary for Wendao.
#[must_use]
pub fn extract_function_args<'a>(contract: &'a str, function_name: &str) -> Option<&'a str> {
    crate::parsers::semantic_check::extract_function_args(contract, function_name)
}
/// `generate_suggested_id` public function boundary for Wendao.
#[must_use]
pub fn generate_suggested_id(title: &str) -> String {
    crate::parsers::semantic_check::generate_suggested_id(title)
}
/// `xml_escape` public function boundary for Wendao.
#[must_use]
pub fn xml_escape(s: &str) -> String {
    super::report::xml_escape(s)
}
/// `issue_type_to_code` public function boundary for Wendao.
#[must_use]
pub fn issue_type_to_code(issue_type: &str) -> &'static str {
    super::report::issue_type_to_code(issue_type)
}
/// `build_file_reports` public function boundary for Wendao.
#[must_use]
pub fn build_file_reports(issues: &[SemanticIssue], docs: &[String]) -> Vec<FileAuditReport> {
    super::report::build_file_reports(issues, docs)
}
/// `format_result_as_xml` public function boundary for Wendao.
#[must_use]
pub fn format_result_as_xml(result: &SemanticCheckResult) -> String {
    super::report::format_result_as_xml(result)
}
/// `check_code_observations` public function boundary for Wendao.
/// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn check_code_observations(
    node: &PageIndexNode,
    doc_id: &str,
    source_files: &[SourceFile],
    fuzzy_threshold: Option<f32>,
    issues: &mut Vec<SemanticIssue>,
) {
    super::checks::check_code_observations(node, doc_id, source_files, fuzzy_threshold, issues);
}
