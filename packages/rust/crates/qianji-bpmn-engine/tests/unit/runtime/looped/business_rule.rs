use super::super::{StubHost, dmn_fixture_definition, standard_loop_business_rule_process};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnPackage, advance_instance, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_standard_loop_business_rule_executes_locally_until_loop_maximum() {
    let package = Arc::new(
        BpmnPackage::new(
            "pkg_runtime",
            vec![standard_loop_business_rule_process(
                "loop_local_business_rule",
            )],
        )
        .with_dmn_decisions(vec![dmn_fixture_definition(
            "simple-unique-eligibility.dmn",
        )]),
    );
    let mut instance = create_instance(
        Arc::clone(&package),
        "loop_local_business_rule",
        BpmnInstanceInit::new("wf_loop_local_business_rule", json!({ "tier": "gold" }), 10),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(190))
        .await
        .must("local business-rule loop should complete without host work");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.standard_loops.is_empty());
    assert_eq!(
        instance.variables,
        json!({ "tier": "gold", "approval": "approve" })
    );
    assert_eq!(instance.sequence, 6);
    assert_eq!(instance.updated_at_ms, 190);
}
