use super::{
    Arc, BpmnAdvanceOutcome, BpmnInstanceInit, DmnEvaluationResult, EventPollOutcome,
    InstanceLifecycle, QianjiBpmnHostBridge, QianjiBpmnSession, TempDir, json,
    load_bpmn_package_from_files, ok_of, write_business_rule_bundle, write_event_race_bundle,
    write_wait_bundle,
};

#[tokio::test(flavor = "current_thread")]
async fn run_until_stable_auto_resolves_business_rule_host_work() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bundle = write_business_rule_bundle(&temp_dir);
    let package = ok_of(
        load_bpmn_package_from_files(&bundle.bpmn_path, &[]),
        "bundle should load without local DMN registry",
    );
    let mut session = ok_of(
        QianjiBpmnSession::new(
            Arc::clone(&package),
            "review",
            BpmnInstanceInit::new("wf_review", json!({ "risk": "high" }), 11),
        ),
        "session should be created",
    );
    let host = QianjiBpmnHostBridge::builder()
        .on_business_rule_task(|request| async move {
            Ok(xiuxian_qianji_bpmn_engine::BusinessRuleTaskOutcome {
                evaluation: DmnEvaluationResult::new(
                    request.evaluation.decision.decision_id.as_ref(),
                    json!({ "approved": false, "path": "manual_review" }),
                    vec![Arc::<str>::from("rule_host")],
                ),
            })
        })
        .clock(|| 100)
        .build();

    let outcome = ok_of(
        session.run_until_stable(&host).await,
        "session should resolve host-blocked work automatically",
    );

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(session.instance().lifecycle, InstanceLifecycle::Completed);
    assert_eq!(
        session.instance().variables,
        json!({
            "risk": "high",
            "approved": false,
            "path": "manual_review",
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn run_until_stable_preserves_waiting_when_event_poll_is_unsupported() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let package = ok_of(
        load_bpmn_package_from_files(&bpmn_path, &[]),
        "wait bundle should load from disk",
    );
    let mut session = ok_of(
        QianjiBpmnSession::new(
            Arc::clone(&package),
            "wait_flow",
            BpmnInstanceInit::new("wf_wait", json!({ "amount": 7 }), 11),
        ),
        "session should be created",
    );
    let host = QianjiBpmnHostBridge::default();

    let outcome = ok_of(
        session.run_until_stable(&host).await,
        "unsupported event polling should leave the session waiting",
    );

    assert_eq!(outcome, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(session.instance().lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(session.instance().waits.len(), 1);
    assert_eq!(session.instance().active_tokens.len(), 1);
    assert_eq!(session.instance().active_tokens[0].node_index, 1);
}

#[tokio::test(flavor = "current_thread")]
async fn run_until_stable_auto_resolves_waiting_event_when_ready() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let package = ok_of(
        load_bpmn_package_from_files(&bpmn_path, &[]),
        "wait bundle should load from disk",
    );
    let mut session = ok_of(
        QianjiBpmnSession::new(
            Arc::clone(&package),
            "wait_flow",
            BpmnInstanceInit::new("wf_wait", json!({ "amount": 7 }), 11),
        ),
        "session should be created",
    );
    let host = QianjiBpmnHostBridge::builder()
        .on_event_poll(|request| async move {
            assert_eq!(request.instance_id, "wf_wait");
            assert_eq!(request.waits.len(), 1);
            Ok(EventPollOutcome {
                ready: true,
                winning_wait_node_index: None,
                data: json!({ "approved": true }),
            })
        })
        .clock(|| 144)
        .build();

    let outcome = ok_of(
        session.run_until_stable(&host).await,
        "ready event polling should resume and complete the session",
    );

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(session.instance().lifecycle, InstanceLifecycle::Completed);
    assert!(session.instance().waits.is_empty());
    assert_eq!(
        session.instance().variables,
        json!({
            "amount": 7,
            "approved": true,
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn run_until_stable_auto_resolves_event_competition_winner_when_ready() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_event_race_bundle(&temp_dir);
    let package = ok_of(
        load_bpmn_package_from_files(&bpmn_path, &[]),
        "event-race bundle should load from disk",
    );
    let mut session = ok_of(
        QianjiBpmnSession::new(
            Arc::clone(&package),
            "event_race",
            BpmnInstanceInit::new("wf_event_race", json!({ "amount": 7 }), 11),
        ),
        "session should be created",
    );
    let host = QianjiBpmnHostBridge::builder()
        .on_event_poll(|request| async move {
            assert_eq!(request.instance_id, "wf_event_race");
            assert_eq!(request.gateway_node_index, Some(1));
            assert_eq!(request.waits.len(), 2);
            assert_eq!(
                request
                    .waits
                    .iter()
                    .map(|wait| wait.node_index)
                    .collect::<Vec<_>>(),
                vec![2, 3]
            );
            Ok(EventPollOutcome {
                ready: true,
                winning_wait_node_index: Some(2),
                data: json!({
                    "approved": true,
                    "winner": "message",
                }),
            })
        })
        .clock(|| 144)
        .build();

    let outcome = ok_of(
        session.run_until_stable(&host).await,
        "ready event competition should resume and complete the session",
    );

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(session.instance().lifecycle, InstanceLifecycle::Completed);
    assert!(session.instance().waits.is_empty());
    assert!(session.instance().event_competition.is_none());
    assert_eq!(
        session.instance().variables,
        json!({
            "amount": 7,
            "approved": true,
            "winner": "message",
        })
    );
}
