use crate::dmn_model_api::{
    DmnDecisionRef, DmnHitPolicy, DmnInputClause, DmnInputEntry, DmnOutputClause, DmnOutputEntry,
};
use serde_json::Value;
use std::sync::Arc;

/// One bounded DMN rule.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnRule {
    /// Stable rule identifier.
    pub rule_id: Arc<str>,
    /// Optional free-form rule description.
    pub description: Option<Arc<str>>,
    /// Input predicates for this rule.
    pub input_entries: Vec<DmnInputEntry>,
    /// Output expressions for this rule.
    pub output_entries: Vec<DmnOutputEntry>,
}

impl DmnRule {
    /// Creates one bounded DMN rule.
    #[must_use]
    pub fn new(
        rule_id: impl AsRef<str>,
        description: Option<impl AsRef<str>>,
        input_entries: Vec<DmnInputEntry>,
        output_entries: Vec<DmnOutputEntry>,
    ) -> Self {
        Self {
            rule_id: Arc::<str>::from(rule_id.as_ref()),
            description: description.map(|value| Arc::<str>::from(value.as_ref())),
            input_entries,
            output_entries,
        }
    }
}

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

/// One bounded DMN decision definition with one decision table.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnDecisionDefinition {
    /// Source identifier used for diagnostics.
    pub source_id: Arc<str>,
    /// Stable decision reference.
    pub decision: DmnDecisionRef,
    /// Optional decision name.
    pub name: Option<Arc<str>>,
    /// The single bounded decision table.
    pub table: DmnDecisionTable,
}

impl DmnDecisionDefinition {
    /// Creates one bounded decision definition.
    #[must_use]
    pub fn new(
        source_id: impl AsRef<str>,
        decision: DmnDecisionRef,
        name: Option<impl AsRef<str>>,
        table: DmnDecisionTable,
    ) -> Self {
        Self {
            source_id: Arc::<str>::from(source_id.as_ref()),
            decision,
            name: name.map(|value| Arc::<str>::from(value.as_ref())),
            table,
        }
    }

    /// Returns whether the provided reference matches this parsed decision.
    #[must_use]
    pub fn matches_reference(&self, other: &DmnDecisionRef) -> bool {
        self.decision.decision_id == other.decision_id
            && other
                .source_id
                .as_deref()
                .is_none_or(|source_id| source_id == self.source_id.as_ref())
    }
}

/// DMN evaluation request surface.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnEvaluationRequest {
    /// Target decision reference.
    pub decision: DmnDecisionRef,
    /// Input variables supplied by the host.
    pub variables: Value,
}

impl DmnEvaluationRequest {
    /// Creates one DMN evaluation request.
    #[must_use]
    pub fn new(decision: DmnDecisionRef, variables: Value) -> Self {
        Self {
            decision,
            variables,
        }
    }
}

/// DMN evaluation result surface.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnEvaluationResult {
    /// Evaluated decision identity.
    pub decision_id: Arc<str>,
    /// Output payload.
    pub output: Value,
    /// Rule identifiers that matched during evaluation.
    pub matched_rule_ids: Vec<Arc<str>>,
}

impl DmnEvaluationResult {
    /// Creates one DMN evaluation result.
    #[must_use]
    pub fn new(
        decision_id: impl AsRef<str>,
        output: Value,
        matched_rule_ids: Vec<Arc<str>>,
    ) -> Self {
        Self {
            decision_id: Arc::<str>::from(decision_id.as_ref()),
            output,
            matched_rule_ids,
        }
    }
}
