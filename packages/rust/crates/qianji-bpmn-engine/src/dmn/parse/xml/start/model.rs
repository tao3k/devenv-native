use super::{
    TempContextEntry, TempContextExpression, TempDecision, TempInvocation, TempInvocationBinding,
    TempListExpression, TempLiteralExpression, TempRelationExpression, TempRelationRow, TempTable,
};

pub(super) struct DecisionStartScope<'a> {
    pub(super) current_decision: &'a mut Option<TempDecision>,
    pub(super) current_literal: &'a mut Option<TempLiteralExpression>,
    pub(super) current_list: &'a mut Option<TempListExpression>,
    pub(super) current_context: &'a mut Option<TempContextExpression>,
    pub(super) current_context_entry: &'a mut Option<TempContextEntry>,
    pub(super) current_relation: &'a mut Option<TempRelationExpression>,
    pub(super) current_relation_row: &'a mut Option<TempRelationRow>,
    pub(super) current_invocation: &'a mut Option<TempInvocation>,
    pub(super) current_invocation_binding: &'a mut Option<TempInvocationBinding>,
    pub(super) current_table: &'a mut Option<TempTable>,
    pub(super) parent_tag: Option<&'a str>,
}

#[derive(Clone, Copy)]
pub(super) struct SurfaceStartState<'a> {
    pub(super) decision: Option<&'a TempDecision>,
    pub(super) literal: Option<&'a TempLiteralExpression>,
    pub(super) invocation: Option<&'a TempInvocation>,
    pub(super) table: Option<&'a TempTable>,
}

impl<'a> SurfaceStartState<'a> {
    pub(super) fn new(
        decision: Option<&'a TempDecision>,
        literal: Option<&'a TempLiteralExpression>,
        invocation: Option<&'a TempInvocation>,
        table: Option<&'a TempTable>,
    ) -> Self {
        Self {
            decision,
            literal,
            invocation,
            table,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct PeerSurfaceState<'a> {
    pub(super) list: Option<&'a TempListExpression>,
    pub(super) context: Option<&'a TempContextExpression>,
    pub(super) relation: Option<&'a TempRelationExpression>,
}

impl<'a> PeerSurfaceState<'a> {
    pub(super) fn new(
        list: Option<&'a TempListExpression>,
        context: Option<&'a TempContextExpression>,
        relation: Option<&'a TempRelationExpression>,
    ) -> Self {
        Self {
            list,
            context,
            relation,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct BoxedExpressionPeerState<'a> {
    pub(super) list: Option<&'a TempListExpression>,
    pub(super) context: Option<&'a TempContextExpression>,
    pub(super) relation: Option<&'a TempRelationExpression>,
    pub(super) invocation: Option<&'a TempInvocation>,
}

impl<'a> BoxedExpressionPeerState<'a> {
    pub(super) fn new(
        list: Option<&'a TempListExpression>,
        context: Option<&'a TempContextExpression>,
        relation: Option<&'a TempRelationExpression>,
        invocation: Option<&'a TempInvocation>,
    ) -> Self {
        Self {
            list,
            context,
            relation,
            invocation,
        }
    }
}

pub(super) struct DecisionChildStartScope<'a> {
    pub(super) decision: Option<&'a TempDecision>,
    pub(super) literal: &'a mut Option<TempLiteralExpression>,
    pub(super) list: &'a mut Option<TempListExpression>,
    pub(super) context: &'a mut Option<TempContextExpression>,
    pub(super) context_entry: &'a mut Option<TempContextEntry>,
    pub(super) relation: &'a mut Option<TempRelationExpression>,
    pub(super) relation_row: &'a mut Option<TempRelationRow>,
    pub(super) invocation: &'a mut Option<TempInvocation>,
    pub(super) invocation_binding: &'a mut Option<TempInvocationBinding>,
    pub(super) table: &'a mut Option<TempTable>,
}

pub(super) struct DirectDecisionSurfaceStartScope<'a> {
    pub(super) decision: Option<&'a TempDecision>,
    pub(super) literal: &'a mut Option<TempLiteralExpression>,
    pub(super) list: &'a mut Option<TempListExpression>,
    pub(super) context: &'a mut Option<TempContextExpression>,
    pub(super) relation: &'a mut Option<TempRelationExpression>,
    pub(super) invocation: &'a mut Option<TempInvocation>,
    pub(super) invocation_binding: &'a mut Option<TempInvocationBinding>,
    pub(super) table: Option<&'a TempTable>,
}

pub(super) struct InvocationChildStartScope<'a> {
    pub(super) literal: &'a mut Option<TempLiteralExpression>,
    pub(super) invocation: &'a mut Option<TempInvocation>,
    pub(super) binding: &'a mut Option<TempInvocationBinding>,
}

pub(super) struct ContextChildStartScope<'a> {
    pub(super) literal: &'a mut Option<TempLiteralExpression>,
    pub(super) context: Option<&'a TempContextExpression>,
    pub(super) entry: &'a mut Option<TempContextEntry>,
}

pub(super) struct RelationChildStartScope<'a> {
    pub(super) literal: &'a mut Option<TempLiteralExpression>,
    pub(super) relation: Option<&'a mut TempRelationExpression>,
    pub(super) row: &'a mut Option<TempRelationRow>,
}
