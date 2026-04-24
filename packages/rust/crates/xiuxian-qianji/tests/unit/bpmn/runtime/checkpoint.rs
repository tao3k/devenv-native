#![cfg(feature = "duckdb")]

use super::*;

#[tokio::test(flavor = "current_thread")]
async fn duckdb_checkpoint_store_round_trips_session_checkpoint() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bundle = write_business_rule_bundle(&temp_dir);
    let package = ok_of(
        load_bpmn_package_from_files(&bundle.bpmn_path, std::slice::from_ref(&bundle.dmn_path)),
        "bundle should load from disk",
    );
    let session = ok_of(
        QianjiBpmnSession::new(
            Arc::clone(&package),
            "review",
            BpmnInstanceInit::new("wf_duckdb", json!({ "risk": "low" }), 5),
        ),
        "session should be created",
    );
    let store = QianjiBpmnCheckpointStore::duckdb(temp_dir.path().join("bpmn.duckdb"));

    ok_of(
        session.save_checkpoint(&store).await,
        "checkpoint should save to duckdb store",
    );
    let resumed = ok_of(
        QianjiBpmnSession::load_from_store(Arc::clone(&package), "wf_duckdb", &store).await,
        "checkpoint load should succeed",
    )
    .unwrap_or_else(|| panic!("checkpoint should exist in duckdb store"));

    assert_eq!(resumed.instance().instance_id.as_ref(), "wf_duckdb");
    assert_eq!(resumed.instance().variables, json!({ "risk": "low" }));
    assert_eq!(resumed.instance().process.process_id.as_ref(), "review");
}

#[tokio::test(flavor = "current_thread")]
async fn execution_driver_resumes_checkpointed_session_from_duckdb_store() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let package = ok_of(
        load_bpmn_package_from_files(&bpmn_path, &[]),
        "wait bundle should load from disk",
    );
    let store = QianjiBpmnCheckpointStore::duckdb(temp_dir.path().join("bpmn.duckdb"));
    let mut seeded_session = ok_of(
        QianjiBpmnSession::new(
            Arc::clone(&package),
            "wait_flow",
            BpmnInstanceInit::new("wf_driver_resume", json!({ "amount": 7 }), 11),
        ),
        "session should be created",
    );

    let waiting = ok_of(
        seeded_session
            .run_until_stable(&QianjiBpmnHostBridge::default())
            .await,
        "unsupported event polling should leave the seeded session waiting",
    );
    assert_eq!(waiting, BpmnAdvanceOutcome::WaitingExternalEvent);
    ok_of(
        seeded_session.save_checkpoint(&store).await,
        "seeded waiting session should save to duckdb store",
    );

    let driver = QianjiBpmnExecutionDriver::new(Arc::clone(&package), Some(store));
    let host = QianjiBpmnHostBridge::builder()
        .on_event_poll(|request| async move {
            assert_eq!(request.instance_id, "wf_driver_resume");
            Ok(EventPollOutcome {
                ready: true,
                winning_wait_node_index: None,
                data: json!({ "approved": true }),
            })
        })
        .clock(|| 144)
        .build();

    let execution = ok_of(
        driver
            .run_until_stable(
                &QianjiBpmnExecutionRequest::new("wait_flow", "wf_driver_resume", None, 17),
                &host,
            )
            .await,
        "driver should resume the stored waiting session",
    );

    assert_eq!(execution.outcome, BpmnAdvanceOutcome::Completed);
    assert!(execution.resumed_from_checkpoint);
    assert!(execution.checkpoint_saved);
    assert!(!execution.checkpoint_deleted);
    assert_eq!(
        execution.session.instance().variables,
        json!({
            "amount": 7,
            "approved": true,
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn execution_driver_skips_checkpoint_save_when_resumed_wait_stays_stable() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bpmn_path = write_wait_bundle(&temp_dir);
    let package = ok_of(
        load_bpmn_package_from_files(&bpmn_path, &[]),
        "wait bundle should load from disk",
    );
    let store = QianjiBpmnCheckpointStore::duckdb(temp_dir.path().join("bpmn.duckdb"));
    let mut seeded_session = ok_of(
        QianjiBpmnSession::new(
            Arc::clone(&package),
            "wait_flow",
            BpmnInstanceInit::new("wf_driver_wait", json!({ "amount": 7 }), 11),
        ),
        "session should be created",
    );

    let waiting = ok_of(
        seeded_session
            .run_until_stable(&QianjiBpmnHostBridge::default())
            .await,
        "unsupported event polling should leave the seeded session waiting",
    );
    assert_eq!(waiting, BpmnAdvanceOutcome::WaitingExternalEvent);
    let original_sequence = seeded_session.instance().sequence;
    ok_of(
        seeded_session.save_checkpoint(&store).await,
        "seeded waiting session should save to duckdb store",
    );

    let driver = QianjiBpmnExecutionDriver::new(Arc::clone(&package), Some(store));
    let execution = ok_of(
        driver
            .run_until_stable(
                &QianjiBpmnExecutionRequest::new("wait_flow", "wf_driver_wait", None, 17),
                &QianjiBpmnHostBridge::default(),
            )
            .await,
        "driver should resume the stored waiting session without new progress",
    );

    assert_eq!(execution.outcome, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert!(execution.resumed_from_checkpoint);
    assert!(!execution.checkpoint_saved);
    assert!(!execution.checkpoint_deleted);
    assert_eq!(execution.session.instance().sequence, original_sequence);
}
