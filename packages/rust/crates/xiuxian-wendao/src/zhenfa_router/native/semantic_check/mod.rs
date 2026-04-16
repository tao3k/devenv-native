//! Semantic Check Tool (Blueprint v2.0 Section 3: Project Sentinel).
//!
//! This module implements the "Semantic Sentinel" concept:
//! - Dead link detection: Scan all `[[id]]` references and verify against global ID index
//! - Status sentinel: Report references to DEPRECATED nodes
//! - Contract validation: Check `:CONTRACT:` constraints
//! - Code observation validation: Check `:OBSERVE:` patterns (Blueprint v2.7)
//! - Fuzzy pattern suggestion: Suggest fixes for invalid patterns (Blueprint v2.9)
//! - Docs governance validation for package-local crate docs

mod checks;
mod core;
mod report;
mod types;

pub mod docs_governance;
#[doc(hidden)]
pub mod test_api;

pub use core::{WendaoSemanticCheckTool, run_audit_core, wendao_semantic_check};
pub use types::{
    CheckType, FileAuditReport, FuzzySuggestionData, HashReference, IssueLocation, NodeStatus,
    SemanticCheckResult, SemanticIssue, WendaoSemanticCheckArgs,
};

#[cfg(test)]
pub(crate) use test_api::SourceFile;

#[cfg(test)]
pub(crate) use test_api::check_code_observations;
#[cfg(test)]
pub(crate) use test_api::{build_file_reports, issue_type_to_code, xml_escape};
#[cfg(test)]
pub(crate) use test_api::{
    extract_function_args, extract_hash_references, extract_id_references, generate_suggested_id,
    validate_contract,
};

#[cfg(test)]
#[path = "../../../../tests/unit/semantic_check_tests/mod.rs"]
mod tests;
