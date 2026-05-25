//! Public dmn parse api contracts for BPMN/DMN engine integration.

#[path = "dmn/parse/mod.rs"]
mod parser;

use crate::{BpmnEngineError, DmnDecisionDefinition, DmnSourceFile};
type Result<T> = std::result::Result<T, BpmnEngineError>;

pub(crate) use parser::parse_literal;

/// Parses one bounded DMN source into one or more decision definitions.
///
/// # Errors
///
/// Returns typed DMN parse errors when the XML payload is malformed or when
/// any parsed decision exceeds the bounded one-table and current hit-policy
/// slice.
pub fn parse_dmn_decisions(source: &DmnSourceFile) -> Result<Vec<DmnDecisionDefinition>> {
    parser::parse_dmn_decisions_impl(source)
}

/// Parses one bounded DMN source into a single-decision definition.
///
/// # Errors
///
/// Returns typed DMN parse errors when the XML payload is malformed, when the
/// source contains no decisions, or when the source contains anything other
/// than exactly one bounded decision.
pub fn parse_dmn_decision(source: &DmnSourceFile) -> Result<DmnDecisionDefinition> {
    parser::parse_dmn_decision_impl(source)
}
