use crate::test_support::{MustExt as _, data_object_io_bpmn};
use qianji_bpmn_engine::{
    BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnInstanceInit, BpmnNodeKind, BpmnNodeSpec,
    BpmnPackage, BpmnParseOptions, BpmnProcessSpec, BpmnScriptTaskSpec, BpmnSourceFile,
    BpmnTaskInputBinding, BpmnTaskInputSource, BpmnTaskIoSpec, BpmnTaskOutputBinding,
    DmnDecisionRef, PendingHostWorkRequest, PendingHostWorkResult, ProcessKey, ServiceTaskOutcome,
    advance_instance, build_pending_host_work_request, create_instance, parse_bpmn_package,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_service_request_materializes_task_io_metadata() {
    let process = BpmnProcessSpec::new(
        ProcessKey::new("pkg_dispatch", "dispatch_inputs", "digest_dispatch_inputs"),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "task", BpmnNodeKind::ServiceTask).with_task_io(
                BpmnTaskIoSpec::new()
                    .with_input(BpmnTaskInputBinding::new(
                        "amount",
                        BpmnTaskInputSource::variable("order.amount"),
                    ))
                    .with_input(BpmnTaskInputBinding::new(
                        "mode",
                        BpmnTaskInputSource::literal(r#"{"priority":"fast"}"#),
                    ))
                    .with_output(BpmnTaskOutputBinding::new("approval", "review.approval")),
            ),
            BpmnNodeSpec::new(2, "end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 2, None::<&str>),
        ],
        Vec::new(),
    );
    let package = Arc::new(BpmnPackage::new("pkg_dispatch", vec![process]));
    let mut instance = create_instance(
        Arc::clone(&package),
        "dispatch_inputs",
        BpmnInstanceInit::new(
            "wf_dispatch_inputs",
            json!({ "order": { "amount": 7 } }),
            10,
        ),
    )
    .must("instance should be created");
    let host = super::support::StubHost::new(55);

    advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("initial advance should block on host work");
    let request =
        build_pending_host_work_request(&instance).must("blocked instance should emit request");
    let PendingHostWorkRequest::Service(request) = request else {
        panic!("expected service request");
    };

    assert_eq!(
        request.inputs,
        json!({ "amount": 7, "mode": { "priority": "fast" } })
    );
    assert_eq!(
        request.output_bindings,
        vec![BpmnTaskOutputBinding::new("approval", "review.approval")]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_and_resume_copy_through_data_object_reference_associations() {
    let package = Arc::new(
        parse_bpmn_package(
            &[BpmnSourceFile::new(
                "service-task-data-object-io.bpmn",
                data_object_io_bpmn(),
            )],
            &BpmnParseOptions::default(),
        )
        .must("data object IO BPMN should parse"),
    );
    let mut instance = create_instance(
        Arc::clone(&package),
        "service_task_data_object_io",
        BpmnInstanceInit::new(
            "wf_data_object_io",
            json!({ "OrderData": { "amount": 7 } }),
            10,
        ),
    )
    .must("instance should be created");
    let host = super::support::StubHost::new(55);

    advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("initial advance should block on host work");
    let request =
        build_pending_host_work_request(&instance).must("blocked instance should emit request");
    let PendingHostWorkRequest::Service(request) = request else {
        panic!("expected service request");
    };

    assert_eq!(request.inputs, json!({ "order": { "amount": 7 } }));
    assert_eq!(
        request.output_bindings,
        vec![BpmnTaskOutputBinding::new("decision", "OrderData")]
    );
    let token_id = request.token_id;

    crate::test_support::apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::Service(ServiceTaskOutcome {
            data: json!({ "decision": { "approved": true } }),
        }),
        100,
    )
    .must("data object target should receive mapped completion data");

    assert_eq!(instance.variables["OrderData"], json!({ "approved": true }));
}

#[tokio::test(flavor = "current_thread")]
async fn host_dispatch_all_host_task_requests_materialize_task_io_metadata() {
    for (node_kind, label) in host_task_kinds() {
        let process_id = format!("dispatch_inputs_{label}");
        let expected_outputs = vec![BpmnTaskOutputBinding::new("approval", "review.approval")];
        let process = host_task_process(
            &process_id,
            &node_kind,
            BpmnTaskIoSpec::new()
                .with_input(BpmnTaskInputBinding::new(
                    "amount",
                    BpmnTaskInputSource::variable("order.amount"),
                ))
                .with_input(BpmnTaskInputBinding::new(
                    "mode",
                    BpmnTaskInputSource::literal(r#"{"priority":"fast"}"#),
                ))
                .with_output(BpmnTaskOutputBinding::new("approval", "review.approval")),
        );
        let package = Arc::new(BpmnPackage::new("pkg_dispatch", vec![process]));
        let mut instance = create_instance(
            Arc::clone(&package),
            &process_id,
            BpmnInstanceInit::new(
                format!("wf_dispatch_inputs_{label}"),
                json!({ "order": { "amount": 7 } }),
                10,
            ),
        )
        .must("instance should be created");
        let host = super::support::StubHost::new(55);

        advance_instance(package.as_ref(), &mut instance, &host)
            .await
            .must("initial advance should block on host work");
        let request =
            build_pending_host_work_request(&instance).must("blocked instance should emit request");
        let (inputs, output_bindings) = request_io_metadata(&request);

        assert_eq!(
            inputs,
            json!({ "amount": 7, "mode": { "priority": "fast" } })
        );
        assert_eq!(output_bindings, expected_outputs);
        if let PendingHostWorkRequest::BusinessRule(request) = request {
            assert_eq!(
                request.evaluation.variables,
                json!({ "amount": 7, "mode": { "priority": "fast" } })
            );
        }
    }
}

fn host_task_kinds() -> [(BpmnNodeKind, &'static str); 6] {
    [
        (BpmnNodeKind::SendTask, "send"),
        (BpmnNodeKind::ServiceTask, "service"),
        (BpmnNodeKind::ScriptTask, "script"),
        (BpmnNodeKind::BusinessRuleTask, "business_rule"),
        (BpmnNodeKind::UserTask, "user"),
        (BpmnNodeKind::ManualTask, "manual"),
    ]
}

fn host_task_process(
    process_id: &str,
    node_kind: &BpmnNodeKind,
    task_io: BpmnTaskIoSpec,
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
    }
    .with_task_io(task_io);
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

fn request_io_metadata(
    request: &PendingHostWorkRequest,
) -> (serde_json::Value, Vec<BpmnTaskOutputBinding>) {
    match request {
        PendingHostWorkRequest::Send(request) => {
            (request.inputs.clone(), request.output_bindings.clone())
        }
        PendingHostWorkRequest::Service(request) => {
            (request.inputs.clone(), request.output_bindings.clone())
        }
        PendingHostWorkRequest::Script(request) => {
            (request.inputs.clone(), request.output_bindings.clone())
        }
        PendingHostWorkRequest::User(request) => {
            (request.inputs.clone(), request.output_bindings.clone())
        }
        PendingHostWorkRequest::Manual(request) => {
            (request.inputs.clone(), request.output_bindings.clone())
        }
        PendingHostWorkRequest::BusinessRule(request) => {
            (request.inputs.clone(), request.output_bindings.clone())
        }
    }
}
