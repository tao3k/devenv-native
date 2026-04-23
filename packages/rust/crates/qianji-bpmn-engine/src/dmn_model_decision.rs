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

/// One bounded direct DMN literal expression.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnLiteralExpression {
    /// Stable literal-expression identifier when present in source.
    pub expression_id: Option<Arc<str>>,
    /// Optional DMN `typeRef` metadata on the expression.
    pub type_ref: Option<Arc<str>>,
    /// Source-level expression body.
    pub text: Arc<str>,
}

impl DmnLiteralExpression {
    /// Creates one bounded literal-expression snapshot.
    #[must_use]
    pub fn new(
        expression_id: Option<impl AsRef<str>>,
        type_ref: Option<impl AsRef<str>>,
        text: impl AsRef<str>,
    ) -> Self {
        Self {
            expression_id: expression_id.map(|value| Arc::<str>::from(value.as_ref())),
            type_ref: type_ref.map(|value| Arc::<str>::from(value.as_ref())),
            text: Arc::<str>::from(text.as_ref()),
        }
    }
}

/// One bounded direct DMN list expression.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnListExpression {
    /// Stable list identifier when present in source.
    pub list_id: Option<Arc<str>>,
    /// Ordered direct literal-expression items.
    pub items: Vec<DmnLiteralExpression>,
}

impl DmnListExpression {
    /// Creates one bounded list-expression snapshot.
    #[must_use]
    pub fn new(list_id: Option<impl AsRef<str>>, items: Vec<DmnLiteralExpression>) -> Self {
        Self {
            list_id: list_id.map(|value| Arc::<str>::from(value.as_ref())),
            items,
        }
    }
}

/// One bounded direct DMN context entry.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnContextEntry {
    /// Stable context-entry identifier when present in source.
    pub entry_id: Option<Arc<str>>,
    /// Stable variable identifier when present in source.
    pub variable_id: Option<Arc<str>>,
    /// Optional variable name. A missing name marks the final result entry.
    pub variable_name: Option<Arc<str>>,
    /// Bounded literal-expression body for this context entry.
    pub expression: DmnLiteralExpression,
}

impl DmnContextEntry {
    /// Creates one bounded context-entry snapshot.
    #[must_use]
    pub fn new(
        entry_id: Option<impl AsRef<str>>,
        variable_id: Option<impl AsRef<str>>,
        variable_name: Option<impl AsRef<str>>,
        expression: DmnLiteralExpression,
    ) -> Self {
        Self {
            entry_id: entry_id.map(|value| Arc::<str>::from(value.as_ref())),
            variable_id: variable_id.map(|value| Arc::<str>::from(value.as_ref())),
            variable_name: variable_name.map(|value| Arc::<str>::from(value.as_ref())),
            expression,
        }
    }
}

/// One bounded direct DMN context expression.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnContextExpression {
    /// Stable context identifier when present in source.
    pub context_id: Option<Arc<str>>,
    /// Ordered context entries.
    pub entries: Vec<DmnContextEntry>,
}

impl DmnContextExpression {
    /// Creates one bounded context-expression snapshot.
    #[must_use]
    pub fn new(context_id: Option<impl AsRef<str>>, entries: Vec<DmnContextEntry>) -> Self {
        Self {
            context_id: context_id.map(|value| Arc::<str>::from(value.as_ref())),
            entries,
        }
    }
}

/// One bounded direct DMN relation column.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnRelationColumn {
    /// Stable column identifier.
    pub column_id: Arc<str>,
    /// Optional output name. Falls back to `column_id` when omitted.
    pub name: Option<Arc<str>>,
    /// Optional DMN `typeRef` metadata on the column.
    pub type_ref: Option<Arc<str>>,
}

impl DmnRelationColumn {
    /// Creates one bounded relation-column snapshot.
    #[must_use]
    pub fn new(
        column_id: impl AsRef<str>,
        name: Option<impl AsRef<str>>,
        type_ref: Option<impl AsRef<str>>,
    ) -> Self {
        Self {
            column_id: Arc::<str>::from(column_id.as_ref()),
            name: name.map(|value| Arc::<str>::from(value.as_ref())),
            type_ref: type_ref.map(|value| Arc::<str>::from(value.as_ref())),
        }
    }

    /// Returns the stable output key used for evaluated row objects.
    #[must_use]
    pub fn output_key(&self) -> &str {
        self.name.as_deref().unwrap_or(self.column_id.as_ref())
    }
}

/// One bounded direct DMN relation row.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnRelationRow {
    /// Stable row identifier when present in source.
    pub row_id: Option<Arc<str>>,
    /// Ordered direct literal-expression cell values.
    pub cells: Vec<DmnLiteralExpression>,
}

impl DmnRelationRow {
    /// Creates one bounded relation-row snapshot.
    #[must_use]
    pub fn new(row_id: Option<impl AsRef<str>>, cells: Vec<DmnLiteralExpression>) -> Self {
        Self {
            row_id: row_id.map(|value| Arc::<str>::from(value.as_ref())),
            cells,
        }
    }
}

/// One bounded direct DMN relation expression.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnRelationExpression {
    /// Stable relation identifier when present in source.
    pub relation_id: Option<Arc<str>>,
    /// Ordered direct relation columns.
    pub columns: Vec<DmnRelationColumn>,
    /// Ordered direct relation rows.
    pub rows: Vec<DmnRelationRow>,
}

impl DmnRelationExpression {
    /// Creates one bounded relation-expression snapshot.
    #[must_use]
    pub fn new(
        relation_id: Option<impl AsRef<str>>,
        columns: Vec<DmnRelationColumn>,
        rows: Vec<DmnRelationRow>,
    ) -> Self {
        Self {
            relation_id: relation_id.map(|value| Arc::<str>::from(value.as_ref())),
            columns,
            rows,
        }
    }
}

/// One bounded executable DMN information-requirement reference.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnInformationRequirementReference {
    /// Direct reference element kind, such as `requiredInput`.
    pub reference_kind: Arc<str>,
    /// Direct href placeholder preserved from source.
    pub href: Option<Arc<str>>,
}

impl DmnInformationRequirementReference {
    /// Creates one bounded information-requirement reference.
    #[must_use]
    pub fn new(reference_kind: impl AsRef<str>, href: Option<impl AsRef<str>>) -> Self {
        Self {
            reference_kind: Arc::<str>::from(reference_kind.as_ref()),
            href: href.map(|value| Arc::<str>::from(value.as_ref())),
        }
    }
}

/// One bounded DMN decision definition with one executable decision surface.
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
    /// Optional bounded direct literal-expression decision body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub literal_expression: Option<DmnLiteralExpression>,
    /// Optional bounded direct list decision body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_expression: Option<DmnListExpression>,
    /// Optional bounded direct context decision body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_expression: Option<DmnContextExpression>,
    /// Optional bounded direct relation decision body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation_expression: Option<DmnRelationExpression>,
    /// Direct executable information-requirement href placeholders.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub information_requirements: Vec<DmnInformationRequirementReference>,
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
            literal_expression: None,
            list_expression: None,
            context_expression: None,
            relation_expression: None,
            information_requirements: Vec::new(),
        }
    }

    /// Attaches one bounded direct literal-expression decision body.
    #[must_use]
    pub fn with_literal_expression(mut self, literal_expression: DmnLiteralExpression) -> Self {
        self.literal_expression = Some(literal_expression);
        self
    }

    /// Attaches one bounded direct list decision body.
    #[must_use]
    pub fn with_list_expression(mut self, list_expression: DmnListExpression) -> Self {
        self.list_expression = Some(list_expression);
        self
    }

    /// Attaches one bounded direct context decision body.
    #[must_use]
    pub fn with_context_expression(mut self, context_expression: DmnContextExpression) -> Self {
        self.context_expression = Some(context_expression);
        self
    }

    /// Attaches one bounded direct relation decision body.
    #[must_use]
    pub fn with_relation_expression(mut self, relation_expression: DmnRelationExpression) -> Self {
        self.relation_expression = Some(relation_expression);
        self
    }

    /// Attaches bounded direct information-requirement references.
    #[must_use]
    pub fn with_information_requirements(
        mut self,
        information_requirements: Vec<DmnInformationRequirementReference>,
    ) -> Self {
        self.information_requirements = information_requirements;
        self
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
