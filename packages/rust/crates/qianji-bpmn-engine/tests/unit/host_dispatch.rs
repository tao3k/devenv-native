use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnEngineError, BpmnGatewayKind, BpmnHostBridge,
    BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, BpmnRepeatSpec,
    BpmnSequentialMultiInstanceSpec, BusinessRuleTaskOutcome, BusinessRuleTaskRequest,
    DmnDecisionRef, DmnEvaluationRequest, EventPollOutcome, EventPollRequest, HostBridgeError,
    ManualTaskOutcome, ManualTaskRequest, PendingHostWorkKind, PendingHostWorkRequest, ProcessKey,
    RepeatExecutionContext, SequentialMultiInstanceContext, ServiceTaskOutcome, ServiceTaskRequest,
    UserTaskOutcome, UserTaskRequest, advance_instance, build_pending_host_work_request,
    build_pending_host_work_requests, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_service_request_materializes_from_blocked_instance() {
    assert_dispatch_request(
        BpmnNodeKind::ServiceTask,
        PendingHostWorkRequest::Service(ServiceTaskRequest {
            instance_id: "wf_dispatch".to_string(),
            token_id: 0,
            node_index: 1,
            variables: json!({ "amount": 7 }),
            repeat: None,
        }),
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_user_request_materializes_from_blocked_instance() {
    assert_dispatch_request(
        BpmnNodeKind::UserTask,
        PendingHostWorkRequest::User(UserTaskRequest {
            instance_id: "wf_dispatch".to_string(),
            token_id: 0,
            node_index: 1,
            variables: json!({ "amount": 7 }),
            repeat: None,
        }),
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_manual_request_materializes_from_blocked_instance() {
    assert_dispatch_request(
        BpmnNodeKind::ManualTask,
        PendingHostWorkRequest::Manual(ManualTaskRequest {
            instance_id: "wf_dispatch".to_string(),
            token_id: 0,
            node_index: 1,
            variables: json!({ "amount": 7 }),
            repeat: None,
        }),
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_business_rule_request_materializes_from_blocked_instance() {
    assert_dispatch_request(
        BpmnNodeKind::BusinessRuleTask,
        PendingHostWorkRequest::BusinessRule(BusinessRuleTaskRequest {
            instance_id: "wf_dispatch".to_string(),
            token_id: 0,
            node_index: 1,
            evaluation: DmnEvaluationRequest::new(
                DmnDecisionRef::new("loan-decision"),
                json!({ "amount": 7 }),
            ),
            repeat: None,
        }),
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_sequential_multi_instance_request_includes_repeat_context() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_dispatch",
        vec![blocking_sequential_multi_instance_process(
            "dispatch_multi_instance",
            3,
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "dispatch_multi_instance",
        BpmnInstanceInit::new("wf_dispatch_multi_instance", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("initial advance should block on host work");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));

    let request =
        build_pending_host_work_request(&instance).must("blocked instance should emit request");
    assert_eq!(
        request,
        with_token_id(
            PendingHostWorkRequest::Service(ServiceTaskRequest {
                instance_id: "wf_dispatch_multi_instance".to_string(),
                token_id: 0,
                node_index: 1,
                variables: json!({ "amount": 7 }),
                repeat: Some(RepeatExecutionContext::SequentialMultiInstance(
                    SequentialMultiInstanceContext {
                        iteration_index: 0,
                        total_iterations: 3,
                    },
                )),
            }),
            instance.pending_host_work[0].token_id,
        )
    );
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_requires_pending_work() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_dispatch",
        vec![blocking_process("dispatch", BpmnNodeKind::ServiceTask)],
    ));
    let instance = create_instance(
        Arc::clone(&package),
        "dispatch",
        BpmnInstanceInit::new("wf_dispatch", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");

    let error = build_pending_host_work_request(&instance)
        .must_err("request materialization requires pending host work");

    assert_eq!(
        error,
        BpmnEngineError::MissingPendingHostWork {
            instance_id: "wf_dispatch".to_string(),
        }
    );
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_parallel_host_work_requires_plural_builder() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_dispatch",
        vec![parallel_service_process("dispatch_parallel")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "dispatch_parallel",
        BpmnInstanceInit::new("wf_dispatch_parallel", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("parallel split should block on both host tasks");
    let pending = match blocked {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => pending,
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    };

    assert_eq!(pending.len(), 2);
    assert_eq!(
        build_pending_host_work_requests(&instance)
            .must("parallel blocked instance should emit requests"),
        vec![
            PendingHostWorkRequest::Service(ServiceTaskRequest {
                instance_id: "wf_dispatch_parallel".to_string(),
                token_id: pending[0].token_id,
                node_index: 2,
                variables: json!({ "amount": 7 }),
                repeat: None,
            }),
            PendingHostWorkRequest::Service(ServiceTaskRequest {
                instance_id: "wf_dispatch_parallel".to_string(),
                token_id: pending[1].token_id,
                node_index: 3,
                variables: json!({ "amount": 7 }),
                repeat: None,
            }),
        ]
    );

    let error = build_pending_host_work_request(&instance)
        .must_err("singleton builder should reject multiple pending host works");
    assert_eq!(
        error,
        BpmnEngineError::AmbiguousPendingHostWork {
            instance_id: "wf_dispatch_parallel".to_string(),
            count: 2,
        }
    );
}

async fn assert_dispatch_request(node_kind: BpmnNodeKind, expected: PendingHostWorkRequest) {
    let package = Arc::new(BpmnPackage::new(
        "pkg_dispatch",
        vec![blocking_process("dispatch", node_kind.clone())],
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
            BpmnNodeKind::ServiceTask => PendingHostWorkKind::Service,
            BpmnNodeKind::UserTask => PendingHostWorkKind::User,
            BpmnNodeKind::ManualTask => PendingHostWorkKind::Manual,
            BpmnNodeKind::BusinessRuleTask => PendingHostWorkKind::BusinessRule,
            _ => unreachable!("helper only supports host-driven task kinds"),
        }
    );
}

fn blocking_process(process_id: &str, node_kind: BpmnNodeKind) -> BpmnProcessSpec {
    let task_node = match node_kind {
        BpmnNodeKind::BusinessRuleTask => {
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::BusinessRuleTask)
                .with_decision(DmnDecisionRef::new("loan-decision"))
        }
        _ => BpmnNodeSpec::new(1, "task", node_kind),
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
        Vec::new(),
    )
}

fn parallel_service_process(process_id: &str) -> BpmnProcessSpec {
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

fn blocking_sequential_multi_instance_process(
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

fn with_token_id(expected: PendingHostWorkRequest, token_id: u64) -> PendingHostWorkRequest {
    match expected {
        PendingHostWorkRequest::Service(mut request) => {
            request.token_id = token_id;
            PendingHostWorkRequest::Service(request)
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

struct StubHost {
    now_ms: u64,
}

impl StubHost {
    fn new(now_ms: u64) -> Self {
        Self { now_ms }
    }
}

#[async_trait::async_trait]
impl BpmnHostBridge for StubHost {
    async fn dispatch_service_task(
        &self,
        _request: ServiceTaskRequest,
    ) -> std::result::Result<ServiceTaskOutcome, HostBridgeError> {
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
