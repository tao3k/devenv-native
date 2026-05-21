use crate::runtime::{
    StubHost, dmn_fixture_definition, sequential_multi_instance_data_binding_business_rule_process,
    sequential_multi_instance_data_binding_process,
};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnNodeKind, BpmnPackage, PendingHostWorkResult,
    ServiceTaskOutcome, advance_instance, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_sequential_multi_instance_data_binding_aggregates_array_output() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![sequential_multi_instance_data_binding_process(
            "multi_instance_data_binding",
            BpmnNodeKind::ServiceTask,
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "multi_instance_data_binding",
        BpmnInstanceInit::new(
            "wf_multi_instance_data_binding",
            json!({ "items": [2, 4, 6] }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(281);

    for expected_result in [20, 40, 60] {
        let blocked = advance_instance(package.as_ref(), &mut instance, &host)
            .await
            .must("sequential data-binding iteration should block on host work");
        assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
        let pending = instance
            .pending_host_work
            .first()
            .cloned()
            .must("data-binding iteration should register pending host work");

        crate::test_support::apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending.token_id,
            PendingHostWorkResult::Service(ServiceTaskOutcome {
                data: json!({ "result": expected_result }),
            }),
            300 + u64::try_from(expected_result).must("expected result fits in u64"),
        )
        .must("host completion should capture data-binding output");
    }

    assert_eq!(
        instance.variables,
        json!({
            "items": [2, 4, 6],
            "results": [20, 40, 60],
        })
    );
    assert!(instance.variables.get("item").is_none());
    assert!(instance.variables.get("result").is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_sequential_multi_instance_data_binding_business_rule_uses_iteration_overlay() {
    let package = Arc::new(
        BpmnPackage::new(
            "pkg_runtime",
            vec![
                sequential_multi_instance_data_binding_business_rule_process(
                    "multi_instance_data_binding_business_rule",
                ),
            ],
        )
        .with_dmn_decisions(vec![dmn_fixture_definition(
            "simple-unique-eligibility.dmn",
        )]),
    );
    let mut instance = create_instance(
        Arc::clone(&package),
        "multi_instance_data_binding_business_rule",
        BpmnInstanceInit::new(
            "wf_multi_instance_data_binding_business_rule",
            json!({ "tiers": ["gold", "silver"] }),
            10,
        ),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(282))
        .await
        .must("data-bound business-rule multi-instance should execute locally");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(
        instance.variables,
        json!({
            "tiers": ["gold", "silver"],
            "decisions": ["approve", "review"],
        })
    );
    assert!(instance.variables.get("tier").is_none());
    assert!(instance.variables.get("approval").is_none());
}
