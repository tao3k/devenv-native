use crate::runtime::{StubHost, parallel_multi_instance_data_binding_process};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnNodeKind, BpmnPackage, ServiceTaskOutcome,
    advance_instance, apply_pending_host_work_result, build_pending_host_work_requests,
    create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_parallel_multi_instance_data_binding_aggregates_object_output() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_multi_instance_data_binding_process(
            "parallel_multi_instance_data_binding",
            BpmnNodeKind::ServiceTask,
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_multi_instance_data_binding",
        BpmnInstanceInit::new(
            "wf_parallel_multi_instance_data_binding",
            json!({
                "assignments": {
                    "alpha": "approve",
                    "beta": "review",
                }
            }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(241);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("parallel data-binding path should block on host work");
    let pending = match blocked {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => pending,
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    };
    assert_eq!(pending.len(), 2);

    for pending_work in pending {
        let item = build_pending_host_work_requests(&instance)
            .must("pending requests should still be materializable")
            .into_iter()
            .find_map(|request| match request {
                qianji_bpmn_engine::PendingHostWorkRequest::Service(request)
                    if request.token_id == pending_work.token_id =>
                {
                    request.variables.get("item").cloned()
                }
                _ => None,
            })
            .must("parallel data-bound request should expose its current item");
        apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending_work.token_id,
            qianji_bpmn_engine::PendingHostWorkResult::Service(ServiceTaskOutcome {
                data: json!({
                    "result": format!("{}_done", item.as_str().must("item should be a string")),
                }),
            }),
            320,
        )
        .must("host completion should capture object-shaped data-binding output");
    }

    assert_eq!(
        instance.variables,
        json!({
            "assignments": {
                "alpha": "approve",
                "beta": "review",
            },
            "results": {
                "alpha": "approve_done",
                "beta": "review_done",
            }
        })
    );
    assert!(instance.variables.get("item").is_none());
    assert!(instance.variables.get("result").is_none());
}
