use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnGatewayKind,
    BpmnHostBridge, BpmnInstanceInit, BpmnMultiInstanceDataBindingSpec, BpmnNodeKind, BpmnNodeSpec,
    BpmnPackage, BpmnParallelMultiInstanceSpec, BpmnProcessSpec, BpmnRepeatSpec,
    BpmnScriptTaskSpec, BpmnSequentialMultiInstanceSpec, BusinessRuleTaskOutcome,
    BusinessRuleTaskRequest, DmnDecisionRef, EventPollOutcome, EventPollRequest, HostBridgeError,
    ManualTaskOutcome, ManualTaskRequest, PendingHostWorkKind, PendingHostWorkRequest, ProcessKey,
    ScriptTaskOutcome, ScriptTaskRequest, SendTaskOutcome, SendTaskRequest, ServiceTaskOutcome,
    ServiceTaskRequest, UserTaskOutcome, UserTaskRequest, advance_instance,
    build_pending_host_work_request, create_instance,
};
use serde_json::json;
use std::sync::Arc;

pub(super) async fn assert_dispatch_request(
    node_kind: BpmnNodeKind,
    expected: PendingHostWorkRequest,
) {
    let package = Arc::new(BpmnPackage::new(
        "pkg_dispatch",
        vec![blocking_process("dispatch", &node_kind)],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "dispatch",
        BpmnInstanceInit::new("wf_dispatch", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("initial advance should block on host work");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));

    let request =
        build_pending_host_work_request(&instance).must("blocked instance should emit request");
    let expected = with_token_id(expected, instance.pending_host_work[0].token_id);
    assert_eq!(request, expected);
    assert_eq!(
        request.kind(),
        match node_kind {
            BpmnNodeKind::SendTask => PendingHostWorkKind::Send,
            BpmnNodeKind::ServiceTask => PendingHostWorkKind::Service,
            BpmnNodeKind::ScriptTask => PendingHostWorkKind::Script,
            BpmnNodeKind::UserTask => PendingHostWorkKind::User,
            BpmnNodeKind::ManualTask => PendingHostWorkKind::Manual,
            BpmnNodeKind::BusinessRuleTask => PendingHostWorkKind::BusinessRule,
            _ => unreachable!("helper only supports host-driven task kinds"),
        }
    );
}

pub(super) fn blocking_process(process_id: &str, node_kind: &BpmnNodeKind) -> BpmnProcessSpec {
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
    let events = match node_kind {
        BpmnNodeKind::SendTask => vec![
            BpmnEventSpec::new(1, BpmnEventKind::Message)
                .with_reference_id("invoice_dispatched")
                .with_name("InvoiceDispatched"),
        ],
        _ => Vec::new(),
    };
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_dispatch", process_id, format!("digest_{process_id}")),
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

pub(super) fn parallel_service_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_dispatch", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "fork", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Parallel),
            BpmnNodeSpec::new(2, "left_service", BpmnNodeKind::ServiceTask),
            BpmnNodeSpec::new(3, "right_service", BpmnNodeKind::ServiceTask),
            BpmnNodeSpec::new(4, "join", BpmnNodeKind::Gateway)
                .with_gateway_kind(BpmnGatewayKind::Parallel),
            BpmnNodeSpec::new(5, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, Some("left")),
            BpmnEdgeSpec::new(1, 3, Some("right")),
            BpmnEdgeSpec::new(2, 4, None::<&str>),
            BpmnEdgeSpec::new(3, 4, None::<&str>),
            BpmnEdgeSpec::new(4, 5, None::<&str>),
        ],
        Vec::new(),
    )
}

pub(super) fn blocking_sequential_multi_instance_process(
    process_id: &str,
    loop_cardinality: u32,
) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_dispatch", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::ServiceTask).with_repeat(
                BpmnRepeatSpec::SequentialMultiInstance(BpmnSequentialMultiInstanceSpec::new(
                    loop_cardinality,
                )),
            ),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    )
}

pub(super) fn blocking_parallel_multi_instance_process(
    process_id: &str,
    loop_cardinality: u32,
) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_dispatch", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::ServiceTask).with_repeat(
                BpmnRepeatSpec::ParallelMultiInstance(BpmnParallelMultiInstanceSpec::new(
                    loop_cardinality,
                )),
            ),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    )
}

pub(super) fn blocking_sequential_multi_instance_data_binding_process(
    process_id: &str,
) -> BpmnProcessSpec {
    let binding =
        BpmnMultiInstanceDataBindingSpec::new("items", "item").with_output("results", "result");
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_dispatch", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::ServiceTask).with_repeat(
                BpmnRepeatSpec::SequentialMultiInstance(
                    BpmnSequentialMultiInstanceSpec::from_data_binding(binding),
                ),
            ),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    )
}

pub(super) fn blocking_parallel_multi_instance_data_binding_process(
    process_id: &str,
) -> BpmnProcessSpec {
    let binding =
        BpmnMultiInstanceDataBindingSpec::new("items", "item").with_output("results", "result");
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_dispatch", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::ServiceTask).with_repeat(
                BpmnRepeatSpec::ParallelMultiInstance(
                    BpmnParallelMultiInstanceSpec::from_data_binding(binding),
                ),
            ),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    )
}

pub(super) fn with_token_id(
    expected: PendingHostWorkRequest,
    token_id: u64,
) -> PendingHostWorkRequest {
    match expected {
        PendingHostWorkRequest::Send(mut request) => {
            request.token_id = token_id;
            PendingHostWorkRequest::Send(request)
        }
        PendingHostWorkRequest::Service(mut request) => {
            request.token_id = token_id;
            PendingHostWorkRequest::Service(request)
        }
        PendingHostWorkRequest::Script(mut request) => {
            request.token_id = token_id;
            PendingHostWorkRequest::Script(request)
        }
        PendingHostWorkRequest::User(mut request) => {
            request.token_id = token_id;
            PendingHostWorkRequest::User(request)
        }
        PendingHostWorkRequest::Manual(mut request) => {
            request.token_id = token_id;
            PendingHostWorkRequest::Manual(request)
        }
        PendingHostWorkRequest::BusinessRule(mut request) => {
            request.token_id = token_id;
            PendingHostWorkRequest::BusinessRule(request)
        }
    }
}

pub(super) struct StubHost {
    now_ms: u64,
}

impl StubHost {
    pub(super) fn new(now_ms: u64) -> Self {
        Self { now_ms }
    }
}

#[async_trait::async_trait]
impl BpmnHostBridge for StubHost {
    async fn dispatch_send_task(
        &self,
        _request: SendTaskRequest,
    ) -> std::result::Result<SendTaskOutcome, HostBridgeError> {
        panic!("host dispatch tests should not execute host work");
    }

    async fn dispatch_service_task(
        &self,
        _request: ServiceTaskRequest,
    ) -> std::result::Result<ServiceTaskOutcome, HostBridgeError> {
        panic!("host dispatch tests should not execute host work");
    }

    async fn dispatch_script_task(
        &self,
        _request: ScriptTaskRequest,
    ) -> std::result::Result<ScriptTaskOutcome, HostBridgeError> {
        panic!("host dispatch tests should not execute host work");
    }

    async fn dispatch_user_task(
        &self,
        _request: UserTaskRequest,
    ) -> std::result::Result<UserTaskOutcome, HostBridgeError> {
        panic!("host dispatch tests should not execute host work");
    }

    async fn dispatch_manual_task(
        &self,
        _request: ManualTaskRequest,
    ) -> std::result::Result<ManualTaskOutcome, HostBridgeError> {
        panic!("host dispatch tests should not execute host work");
    }

    async fn dispatch_business_rule_task(
        &self,
        _request: BusinessRuleTaskRequest,
    ) -> std::result::Result<BusinessRuleTaskOutcome, HostBridgeError> {
        panic!("host dispatch tests should not execute host work");
    }

    async fn poll_external_event(
        &self,
        _request: EventPollRequest,
    ) -> std::result::Result<EventPollOutcome, HostBridgeError> {
        panic!("host dispatch tests should not poll external events");
    }

    fn now_unix_ms(&self) -> u64 {
        self.now_ms
    }
}
