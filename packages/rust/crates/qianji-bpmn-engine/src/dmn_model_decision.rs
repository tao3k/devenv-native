//! Public `api` facade for DMN decision model contracts.

use crate::dmn_model_api::{
    DmnDecisionRef, DmnHitPolicy, DmnInputClause, DmnInputEntry, DmnOutputClause, DmnOutputEntry,
};
use serde_json::Value;
use std::sync::Arc;

mod api;
#[path = "dmn_model_decision/definition/api.rs"]
mod definition;
#[path = "dmn_model_decision/evaluation/api.rs"]
mod evaluation;
#[path = "dmn_model_decision/expression/api.rs"]
mod expression;
#[path = "dmn_model_decision/invocation/api.rs"]
mod invocation;
#[path = "dmn_model_decision/requirement/api.rs"]
mod requirement;
#[path = "dmn_model_decision/rule/api.rs"]
mod rule;
#[path = "dmn_model_decision/table/api.rs"]
mod table;

pub use api::{
    DmnContextEntry, DmnContextExpression, DmnDecisionDefinition, DmnDecisionTable,
    DmnEvaluationRequest, DmnEvaluationResult, DmnInformationRequirementReference, DmnInvocation,
    DmnInvocationBinding, DmnInvocationParameter, DmnKnowledgeRequirementReference,
    DmnListExpression, DmnLiteralExpression, DmnRelationColumn, DmnRelationExpression,
    DmnRelationRow, DmnRule,
};
