use crate::dmn_parse_api::parser::state::{
    CaptureTarget, TempContextEntry, TempContextExpression, TempDecision, TempInput,
    TempInvocation, TempInvocationBinding, TempListExpression, TempLiteralExpression, TempOutput,
    TempRelationExpression, TempRelationRow, TempRule, TempTable,
};

pub(super) struct ParseLoopState {
    pub(super) decisions: Vec<TempDecision>,
    pub(super) current_decision: Option<TempDecision>,
    pub(super) current_literal: Option<TempLiteralExpression>,
    pub(super) current_list: Option<TempListExpression>,
    pub(super) current_context: Option<TempContextExpression>,
    pub(super) current_context_entry: Option<TempContextEntry>,
    pub(super) current_relation: Option<TempRelationExpression>,
    pub(super) current_relation_row: Option<TempRelationRow>,
    pub(super) current_invocation: Option<TempInvocation>,
    pub(super) current_invocation_binding: Option<TempInvocationBinding>,
    pub(super) current_table: Option<TempTable>,
    pub(super) current_input: Option<TempInput>,
    pub(super) current_output: Option<TempOutput>,
    pub(super) current_rule: Option<TempRule>,
    pub(super) capture_target: Option<CaptureTarget>,
    pub(super) capture_buffer: String,
    pub(super) element_stack: Vec<String>,
}

impl ParseLoopState {
    pub(super) fn new() -> Self {
        Self {
            decisions: Vec::new(),
            current_decision: None,
            current_literal: None,
            current_list: None,
            current_context: None,
            current_context_entry: None,
            current_relation: None,
            current_relation_row: None,
            current_invocation: None,
            current_invocation_binding: None,
            current_table: None,
            current_input: None,
            current_output: None,
            current_rule: None,
            capture_target: None,
            capture_buffer: String::new(),
            element_stack: Vec::new(),
        }
    }
}
