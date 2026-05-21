use xiuxian_qianji_control::{
    AgentDecisionId, AgentProposalId, ApprovalRequestId, ApproverId, ControlError,
    DecisionReasonCode, ErrorCode, IdempotencyKey, LlmModelId, PermissionScope, SignalName,
    TaskQueue, TimerId, TokenId, ToolName,
};

#[test]
fn activity_journal_contract_rejects_blank_identity_fields() {
    assert!(matches!(
        TaskQueue::new(" "),
        Err(ControlError::BlankId {
            field: "task_queue"
        })
    ));
    assert!(matches!(
        IdempotencyKey::new(" "),
        Err(ControlError::BlankId {
            field: "idempotency_key"
        })
    ));
    assert!(matches!(
        ErrorCode::new(" "),
        Err(ControlError::BlankId {
            field: "error_code"
        })
    ));
    assert!(matches!(
        LlmModelId::new(" "),
        Err(ControlError::BlankId {
            field: "llm_model_id"
        })
    ));
    assert!(matches!(
        ApprovalRequestId::new(" "),
        Err(ControlError::BlankId {
            field: "approval_request_id"
        })
    ));
    assert!(matches!(
        ApproverId::new(" "),
        Err(ControlError::BlankId {
            field: "approver_id"
        })
    ));
    assert!(matches!(
        AgentProposalId::new(" "),
        Err(ControlError::BlankId {
            field: "agent_proposal_id"
        })
    ));
    assert!(matches!(
        AgentDecisionId::new(" "),
        Err(ControlError::BlankId {
            field: "agent_decision_id"
        })
    ));
    assert!(matches!(
        DecisionReasonCode::new(" "),
        Err(ControlError::BlankId {
            field: "decision_reason_code"
        })
    ));
    assert!(matches!(
        TokenId::new(" "),
        Err(ControlError::BlankId { field: "token_id" })
    ));
    assert!(matches!(
        ToolName::new(" "),
        Err(ControlError::BlankId { field: "tool_name" })
    ));
    assert!(matches!(
        PermissionScope::new(" "),
        Err(ControlError::BlankId {
            field: "permission_scope"
        })
    ));
    assert!(matches!(
        SignalName::new(" "),
        Err(ControlError::BlankId {
            field: "signal_name"
        })
    ));
    assert!(matches!(
        TimerId::new(" "),
        Err(ControlError::BlankId { field: "timer_id" })
    ));
}
