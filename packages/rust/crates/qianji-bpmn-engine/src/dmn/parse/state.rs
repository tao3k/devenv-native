//! Canonical api seam for DMN XML parse temporary state and finalizers.

#[path = "state/api.rs"]
mod api;
#[path = "state/boxed_expression.rs"]
mod boxed_expression;
#[path = "state/decision.rs"]
mod decision;
#[path = "state/model.rs"]
mod model;
#[path = "state/requirement.rs"]
mod requirement;
#[path = "state/table.rs"]
mod table;

pub(crate) use api::{
    CaptureTarget, TempContextEntry, TempContextExpression, TempDecision,
    TempInformationRequirementReference, TempInput, TempInvocation, TempInvocationBinding,
    TempInvocationParameter, TempKnowledgeRequirementReference, TempListExpression,
    TempLiteralExpression, TempOutput, TempRelationColumn, TempRelationExpression, TempRelationRow,
    TempRule, TempTable, finalize_decision_definitions, finalize_input, finalize_input_entry,
    finalize_output, finalize_output_entry, finalize_rule, hit_policy_from_attr,
};
