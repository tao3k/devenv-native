#[path = "dmn/parse.rs"]
mod parser;

use crate::dmn_model_api::{DmnDecisionDefinition, DmnSourceFile};
use crate::error::Result;

/// Parses one bounded DMN source into a single-decision definition.
///
/// # Errors
///
/// Returns typed DMN parse errors when the XML payload is malformed or when
/// the document exceeds the bounded single-decision and single-table slice.
pub fn parse_dmn_decision(source: &DmnSourceFile) -> Result<DmnDecisionDefinition> {
    parser::parse_dmn_decision_impl(source)
}
