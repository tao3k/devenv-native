//! DMN XML start-element dispatcher.
//!
//! This branch coordinates parser state, DMN model contracts, XML decoding,
//! and engine errors so each expression-specific parser leaf can stay focused
//! on one start-tag family.

mod api;
mod context;
mod decision;
mod invocation;
mod list;
mod model;
mod relation;
mod result;
mod table;

use context::{handle_context_child_start_tag, start_context_expression};
use decision::handle_decision_start_tag;
use invocation::{handle_invocation_child_start_tag, start_invocation_expression};
use list::{handle_list_child_start_tag, start_list_expression};
use model::{
    BoxedExpressionPeerState, ContextChildStartScope, DecisionChildStartScope, DecisionStartScope,
    DirectDecisionSurfaceStartScope, InvocationChildStartScope, PeerSurfaceState,
    RelationChildStartScope, SurfaceStartState,
};
use relation::{handle_relation_child_start_tag, start_relation_expression};
use table::{
    handle_capture_start_tag, handle_input_expression_start_tag,
    handle_literal_expression_text_start_tag, handle_table_start_tag, start_decision_table,
};

pub(crate) use api::handle_start_tag;

pub(super) use super::{attribute_value, local_name, required_attribute};
pub(super) use crate::BpmnEngineError;
pub(super) use crate::DmnSourceFile;
pub(super) use crate::dmn_parse_api::parser::state::{
    CaptureTarget, TempContextEntry, TempContextExpression, TempDecision,
    TempInformationRequirementReference, TempInput, TempInvocation, TempInvocationBinding,
    TempInvocationParameter, TempKnowledgeRequirementReference, TempListExpression,
    TempLiteralExpression, TempOutput, TempRelationColumn, TempRelationExpression, TempRelationRow,
    TempRule, TempTable, finalize_input, finalize_input_entry, finalize_output,
    finalize_output_entry, finalize_rule, hit_policy_from_attr,
};
pub(super) use quick_xml::Reader;
pub(super) use quick_xml::events::BytesStart;
pub(super) use result::Result;
