use super::StubHost;
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnParseOptions, BpmnSourceFile, PendingHostWorkRequest,
    advance_instance, build_pending_host_work_request, create_instance, parse_bpmn_package,
};

#[tokio::test(flavor = "current_thread")]
async fn parsed_bpmn_lane_membership_projects_to_pending_human_work() {
    let package = parse_bpmn_package(
        &[fixture_source("human-task-lane.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("BPMN with passive lane metadata should parse");
    let process = package
        .find_process("review")
        .must("review process should exist");
    let review_task = process
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "review_task")
        .must("review task should be normalized");
    let lane = review_task
        .lane
        .as_ref()
        .must("normalized user task should preserve lane membership");
    assert_eq!(lane.set_id.as_deref(), Some("LaneSet_Review"));
    assert_eq!(lane.set_name.as_deref(), Some("Ownership"));
    assert_eq!(lane.id.as_deref(), Some("Lane_Reviewer"));
    assert_eq!(lane.name.as_deref(), Some("Reviewer Lane"));

    let package = Arc::new(package);
    let mut instance = create_instance(
        Arc::clone(&package),
        "review",
        BpmnInstanceInit::new("wf_lane", json!({ "risk": "high" }), 10),
    )
    .must("instance should be created");
    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(11))
        .await
        .must("advance should block on user task");
    let BpmnAdvanceOutcome::BlockedOnHost(pending) = outcome else {
        panic!("advance should block on host work");
    };
    assert_eq!(pending.len(), 1);
    assert_eq!(
        pending[0]
            .lane
            .as_ref()
            .and_then(|lane| lane.name.as_deref()),
        Some("Reviewer Lane")
    );

    let request = build_pending_host_work_request(&instance)
        .must("blocked human work should materialize a host request");
    let PendingHostWorkRequest::User(request) = request else {
        panic!("expected user-task request");
    };
    assert_eq!(
        request.lane.as_ref().and_then(|lane| lane.id.as_deref()),
        Some("Lane_Reviewer")
    );
}

fn fixture_source(name: &str) -> BpmnSourceFile {
    let path = format!("{}/tests/fixtures/bpmn/{name}", env!("CARGO_MANIFEST_DIR"));
    let contents =
        std::fs::read_to_string(path).must("fixture should be readable from the crate tree");
    BpmnSourceFile::new(name, contents)
}
