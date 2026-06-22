use crate::runtime::{
    StubHost, dmn_fixture_definition, sequential_multi_instance_business_rule_process,
};
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnPackage, advance_instance, create_instance,
};

#[tokio::test(flavor = "current_thread")]
async fn runtime_sequential_multi_instance_business_rule_executes_locally() {
    let package = Arc::new(
        BpmnPackage::new(
            "pkg_runtime",
            vec![sequential_multi_instance_business_rule_process(
                "multi_instance_local_business_rule",
                3,
            )],
        )
        .with_dmn_decisions(vec![dmn_fixture_definition(
            "simple-unique-eligibility.dmn",
        )]),
    );
    let mut instance = create_instance(
        Arc::clone(&package),
        "multi_instance_local_business_rule",
        BpmnInstanceInit::new(
            "wf_multi_instance_local_business_rule",
            json!({ "tier": "gold" }),
            10,
        ),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(280))
        .await
        .must("local business-rule multi-instance should complete without host work");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.sequential_multi_instances.is_empty());
    assert_eq!(
        instance.variables,
        json!({ "tier": "gold", "approval": "approve" })
    );
    assert_eq!(instance.sequence, 6);
    assert_eq!(instance.updated_at_ms, 280);
}
