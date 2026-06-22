use crate::dmn_model_api::{
    DmnHitPolicy, DmnInputClause, DmnInputEntry, DmnOutputClause, DmnOutputEntry, DmnRule,
};

pub(crate) struct TempDecision {
    pub(crate) decision_id: String,
    pub(crate) name: Option<String>,
    pub(crate) table: Option<TempTable>,
    pub(crate) literal_expression: Option<TempLiteralExpression>,
    pub(crate) list_expression: Option<TempListExpression>,
    pub(crate) context_expression: Option<TempContextExpression>,
    pub(crate) relation_expression: Option<TempRelationExpression>,
    pub(crate) invocation: Option<TempInvocation>,
    pub(crate) information_requirements: Vec<TempInformationRequirementReference>,
    pub(crate) knowledge_requirements: Vec<TempKnowledgeRequirementReference>,
}

pub(crate) struct TempInformationRequirementReference {
    pub(crate) reference_kind: String,
    pub(crate) href: Option<String>,
}

pub(crate) struct TempKnowledgeRequirementReference {
    pub(crate) reference_kind: String,
    pub(crate) href: Option<String>,
}

pub(crate) struct TempLiteralExpression {
    pub(crate) expression_id: Option<String>,
    pub(crate) type_ref: Option<String>,
    pub(crate) text: Option<String>,
}

pub(crate) struct TempListExpression {
    pub(crate) list_id: Option<String>,
    pub(crate) items: Vec<TempLiteralExpression>,
}

pub(crate) struct TempContextExpression {
    pub(crate) context_id: Option<String>,
    pub(crate) entries: Vec<TempContextEntry>,
}

pub(crate) struct TempContextEntry {
    pub(crate) entry_id: Option<String>,
    pub(crate) variable_id: Option<String>,
    pub(crate) variable_name: Option<String>,
    pub(crate) literal_expression: Option<TempLiteralExpression>,
}

pub(crate) struct TempRelationExpression {
    pub(crate) relation_id: Option<String>,
    pub(crate) columns: Vec<TempRelationColumn>,
    pub(crate) rows: Vec<TempRelationRow>,
}

pub(crate) struct TempRelationColumn {
    pub(crate) column_id: String,
    pub(crate) name: Option<String>,
    pub(crate) type_ref: Option<String>,
}

pub(crate) struct TempRelationRow {
    pub(crate) row_id: Option<String>,
    pub(crate) cells: Vec<TempLiteralExpression>,
}

pub(crate) struct TempInvocation {
    pub(crate) invocation_id: Option<String>,
    pub(crate) invoked_expression: Option<TempLiteralExpression>,
    pub(crate) bindings: Vec<TempInvocationBinding>,
}

pub(crate) struct TempInvocationBinding {
    pub(crate) binding_id: Option<String>,
    pub(crate) parameter: Option<TempInvocationParameter>,
    pub(crate) argument: Option<TempLiteralExpression>,
}

pub(crate) struct TempInvocationParameter {
    pub(crate) parameter_id: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) type_ref: Option<String>,
}

pub(crate) struct TempTable {
    pub(crate) table_id: String,
    pub(crate) name: Option<String>,
    pub(crate) hit_policy: DmnHitPolicy,
    pub(crate) inputs: Vec<DmnInputClause>,
    pub(crate) outputs: Vec<DmnOutputClause>,
    pub(crate) rules: Vec<DmnRule>,
}

pub(crate) struct TempInput {
    pub(crate) input_id: String,
    pub(crate) label: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) expression: Option<String>,
    pub(crate) type_ref: Option<String>,
}

pub(crate) struct TempOutput {
    pub(crate) output_id: String,
    pub(crate) label: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) type_ref: Option<String>,
}

pub(crate) struct TempRule {
    pub(crate) rule_id: String,
    pub(crate) description: Option<String>,
    pub(crate) input_entries: Vec<DmnInputEntry>,
    pub(crate) output_entries: Vec<DmnOutputEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CaptureTarget {
    InputExpression,
    LiteralExpression,
    RuleDescription,
    InputEntry,
    OutputEntry,
}
