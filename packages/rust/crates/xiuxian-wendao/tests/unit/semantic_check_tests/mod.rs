//! Unit tests for `semantic_check` module (Blueprint v2.2).

pub(super) use super::{
    EpistemeLoadReport, EpistemePolicyQueryReport, NodeStatus, SemanticCheckResult, SemanticIssue,
    SourceFile, build_file_reports, check_code_observations, extract_function_args,
    extract_hash_references, extract_id_references, format_result_as_xml, generate_suggested_id,
    issue_type_to_code, validate_contract, xml_escape,
};
pub(super) use crate::link_graph::{PageIndexMeta, PageIndexNode};
pub(super) use crate::parsers::markdown::CodeObservation;
pub(super) use std::sync::Arc;

mod contract_validation;
mod episteme_report;
mod extraction;
mod health_score;
mod helper_functions;
mod observation_checks;
mod support;
