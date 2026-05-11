//! Public `api` facade for DMN decision model contracts.

use crate::{
    DmnDecisionRef, DmnHitPolicy, DmnInputClause, DmnInputEntry, DmnOutputClause, DmnOutputEntry,
};
use serde_json::Value;
use std::sync::Arc;

mod api;
#[path = "definition/api.rs"]
mod definition;
#[path = "evaluation/api.rs"]
mod evaluation;
#[path = "expression/api.rs"]
mod expression;
#[path = "invocation/api.rs"]
mod invocation;
#[path = "requirement/api.rs"]
mod requirement;
#[path = "rule/api.rs"]
mod rule;
#[path = "table/api.rs"]
mod table;

pub use api::{
    DmnContextEntry, DmnContextExpression, DmnDecisionDefinition, DmnDecisionTable,
    DmnDecisionTableInput, DmnEvaluationRequest, DmnEvaluationResult,
    DmnInformationRequirementReference, DmnInvocation, DmnInvocationBinding,
    DmnInvocationParameter, DmnKnowledgeRequirementReference, DmnListExpression,
    DmnLiteralExpression, DmnRelationColumn, DmnRelationExpression, DmnRelationRow, DmnRule,
};
