use crate::host_resume::support::StubHost;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec,
    BpmnPackage, BpmnProcessSpec, BpmnScriptTaskSpec, BpmnTaskIoSpec, BusinessRuleTaskOutcome,
    DmnDecisionRef, DmnEvaluationResult, ManualTaskOutcome, PendingHostWorkResult, ProcessKey,
    ScriptTaskOutcome, SendTaskOutcome, ServiceTaskOutcome, UserTaskOutcome, advance_instance,
    create_instance,
};
use serde_json::json;
use std::sync::Arc;

pub(super) async fn create_blocked_strict_instance(
    package: Arc<BpmnPackage>,
    process_id: &str,
) -> qianji_bpmn_engine::BpmnInstanceState {
    let mut instance = create_instance(
        Arc::clone(&package),
        process_id,
        BpmnInstanceInit::new("wf_strict_io", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let blocked = advance_instance(package.as_ref(), &mut instance, &StubHost::new(55))
        .await
        .must("initial advance should block on host work");
    assert!(matches!(
        blocked,
        qianji_bpmn_engine::BpmnAdvanceOutcome::BlockedOnHost(_)
    ));
    instance
}

pub(super) fn service_process(process_id: &str, task_node: BpmnNodeSpec) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_resume", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            task_node,
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    )
}

pub(super) fn host_task_kinds() -> [(BpmnNodeKind, &'static str); 6] {
    [
        (BpmnNodeKind::SendTask, "send"),
        (BpmnNodeKind::ServiceTask, "service"),
        (BpmnNodeKind::ScriptTask, "script"),
        (BpmnNodeKind::BusinessRuleTask, "business_rule"),
        (BpmnNodeKind::UserTask, "user"),
        (BpmnNodeKind::ManualTask, "manual"),
    ]
}

pub(super) fn host_task_process(
    process_id: &str,
    node_kind: &BpmnNodeKind,
    task_io: Option<BpmnTaskIoSpec>,
) -> BpmnProcessSpec {
    let task_node = match node_kind {
        BpmnNodeKind::BusinessRuleTask => {
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::BusinessRuleTask)
                .with_decision(DmnDecisionRef::new("loan-decision"))
        }
        BpmnNodeKind::ScriptTask => {
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::ScriptTask).with_script_task(
                BpmnScriptTaskSpec::new(Some("feel"), Some("result = amount + tax")),
            )
        }
        _ => BpmnNodeSpec::new(1, "task", node_kind.clone()),
    };
    let task_node = if let Some(task_io) = task_io {
        task_node.with_task_io(task_io)
    } else {
        task_node
    };
    let events = match node_kind {
        BpmnNodeKind::SendTask => vec![
            BpmnEventSpec::new(1, BpmnEventKind::Message)
                .with_reference_id("invoice_dispatched")
                .with_name("InvoiceDispatched"),
        ],
        _ => Vec::new(),
    };
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_resume", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            task_node,
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        events,
    )
}

pub(super) fn result_for_kind(
    node_kind: &BpmnNodeKind,
    data: serde_json::Value,
) -> PendingHostWorkResult {
    match node_kind {
        BpmnNodeKind::SendTask => PendingHostWorkResult::Send(SendTaskOutcome { data }),
        BpmnNodeKind::ServiceTask => PendingHostWorkResult::Service(ServiceTaskOutcome { data }),
        BpmnNodeKind::ScriptTask => PendingHostWorkResult::Script(ScriptTaskOutcome { data }),
        BpmnNodeKind::BusinessRuleTask => {
            PendingHostWorkResult::BusinessRule(BusinessRuleTaskOutcome {
                evaluation: DmnEvaluationResult::new("loan-decision", data, Vec::<Arc<str>>::new()),
            })
        }
        BpmnNodeKind::UserTask => PendingHostWorkResult::User(UserTaskOutcome { data }),
        BpmnNodeKind::ManualTask => PendingHostWorkResult::Manual(ManualTaskOutcome { data }),
        _ => unreachable!("helper only supports host-dispatched task kinds"),
    }
}
