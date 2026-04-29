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

impl DmnDecisionTable {
    /// Creates one bounded decision table.
    #[must_use]
    pub fn new(
        table_id: impl AsRef<str>,
        name: Option<impl AsRef<str>>,
        hit_policy: DmnHitPolicy,
        inputs: Vec<DmnInputClause>,
        outputs: Vec<DmnOutputClause>,
        rules: Vec<DmnRule>,
    ) -> Self {
        Self {
            table_id: Arc::<str>::from(table_id.as_ref()),
            name: name.map(|value| Arc::<str>::from(value.as_ref())),
            hit_policy,
            inputs,
            outputs,
            rules,
        }
    }
}
