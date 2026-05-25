//! Public dmn model decision table contracts for BPMN/DMN engine integration.

use super::{Arc, DmnHitPolicy, DmnInputClause, DmnOutputClause, DmnRule};

/// One bounded DMN decision table.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnDecisionTable {
    /// Stable decision-table identifier.
    pub table_id: Arc<str>,
    /// Optional table name.
    pub name: Option<Arc<str>>,
    /// Table hit policy.
    pub hit_policy: DmnHitPolicy,
    /// Ordered input clauses.
    pub inputs: Vec<DmnInputClause>,
    /// Ordered output clauses.
    pub outputs: Vec<DmnOutputClause>,
    /// Ordered rules.
    pub rules: Vec<DmnRule>,
}

/// Named construction payload for one bounded decision table.
pub struct DmnDecisionTableInput<'a> {
    /// Stable decision-table identifier.
    pub table_id: &'a str,
    /// Optional table name.
    pub name: Option<&'a str>,
    /// Table hit policy.
    pub hit_policy: DmnHitPolicy,
    /// Ordered input clauses.
    pub inputs: Vec<DmnInputClause>,
    /// Ordered output clauses.
    pub outputs: Vec<DmnOutputClause>,
    /// Ordered rules.
    pub rules: Vec<DmnRule>,
}

impl DmnDecisionTable {
    /// Creates one bounded decision table.
    #[must_use]
    pub fn new(input: DmnDecisionTableInput<'_>) -> Self {
        Self {
            table_id: Arc::<str>::from(input.table_id),
            name: input.name.map(Arc::<str>::from),
            hit_policy: input.hit_policy,
            inputs: input.inputs,
            outputs: input.outputs,
            rules: input.rules,
        }
    }
}
