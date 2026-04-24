#![cfg(feature = "duckdb")]

use super::*;
use std::time::{Duration, Instant};

const CACHED_DUCKDB_CHECKPOINT_PROBE_COUNT: usize = 64;

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
async fn duckdb_checkpoint_store_delete_invalidates_same_process_cache() {
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
            BpmnInstanceInit::new("wf_duckdb_delete_cache", json!({ "risk": "low" }), 5),
        ),
        "session should be created",
    );
    let store = QianjiBpmnCheckpointStore::duckdb(temp_dir.path().join("delete-cache.duckdb"));

    ok_of(
        store.save(&session.checkpoint()).await,
        "checkpoint should save to duckdb store",
    );
    assert!(
        ok_of(
            store.load("wf_duckdb_delete_cache").await,
            "checkpoint should load from same-process cache",
        )
        .is_some()
    );
    ok_of(
        store.delete("wf_duckdb_delete_cache").await,
        "checkpoint should delete from duckdb store",
    );

    assert!(
        ok_of(
            store.load("wf_duckdb_delete_cache").await,
            "deleted checkpoint should not be served from cache",
        )
        .is_none()
    );
}

#[tokio::test(flavor = "current_thread")]
async fn duckdb_checkpoint_store_reopens_from_compacted_latest() {
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
            BpmnInstanceInit::new("wf_duckdb_reopen", json!({ "risk": "high" }), 5),
        ),
        "session should be created",
    );
    let database_path = temp_dir.path().join("reopen-compacted.duckdb");
    let first_store = QianjiBpmnCheckpointStore::duckdb(database_path.clone());

    ok_of(
        first_store.save(&session.checkpoint()).await,
        "checkpoint should save to duckdb event log",
    );

    let reopened_store = QianjiBpmnCheckpointStore::duckdb(database_path);
    let loaded = ok_of(
        reopened_store.load("wf_duckdb_reopen").await,
        "checkpoint should cold-load after reopening duckdb store",
    )
    .unwrap_or_else(|| panic!("reopened checkpoint should exist"));

    assert_eq!(loaded.sequence, session.checkpoint().sequence);
    assert_eq!(loaded.state.variables, json!({ "risk": "high" }));
}

#[tokio::test(flavor = "current_thread")]
async fn duckdb_checkpoint_store_cached_facade_probe_reports_timing() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bundle = write_business_rule_bundle(&temp_dir);
    let package = ok_of(
        load_bpmn_package_from_files(&bundle.bpmn_path, std::slice::from_ref(&bundle.dmn_path)),
        "bundle should load from disk",
    );
    let store = QianjiBpmnCheckpointStore::duckdb(temp_dir.path().join("cached-facade.duckdb"));
    let checkpoints = (0..CACHED_DUCKDB_CHECKPOINT_PROBE_COUNT)
        .map(|index| {
            let sequence = usize_to_u64(index) + 1;
            let session = ok_of(
                QianjiBpmnSession::new(
                    Arc::clone(&package),
                    "review",
                    BpmnInstanceInit::new(
                        format!("wf_cached_duckdb_{index:04}"),
                        json!({
                            "risk": if index % 2 == 0 { "low" } else { "high" },
                            "score": index,
                            "route": format!("lane_{}", index % 8),
                        }),
                        5 + sequence,
                    ),
                ),
                "session should be created",
            );
            session.checkpoint()
        })
        .collect::<Vec<_>>();

    let save_started = Instant::now();
    for checkpoint in &checkpoints {
        ok_of(
            store.save(checkpoint).await,
            "cached DuckDB checkpoint save should succeed",
        );
    }
    let save_elapsed = save_started.elapsed();

    let load_started = Instant::now();
    for checkpoint in &checkpoints {
        let loaded = ok_of(
            store.load(checkpoint.state.instance_id.as_ref()).await,
            "cached DuckDB checkpoint load should succeed",
        )
        .unwrap_or_else(|| panic!("cached DuckDB checkpoint should exist"));
        assert_eq!(loaded.sequence, checkpoint.sequence);
    }
    let load_elapsed = load_started.elapsed();

    println!(
        "qianji duckdb checkpoint-store cached facade perf: instances={} save_ms={:.3} load_ms={:.3} save_avg_us={} load_avg_us={}",
        CACHED_DUCKDB_CHECKPOINT_PROBE_COUNT,
        save_elapsed.as_secs_f64() * 1_000.0,
        load_elapsed.as_secs_f64() * 1_000.0,
        avg_us(save_elapsed, CACHED_DUCKDB_CHECKPOINT_PROBE_COUNT),
        avg_us(load_elapsed, CACHED_DUCKDB_CHECKPOINT_PROBE_COUNT),
    );

    assert!(
        save_elapsed <= cached_facade_budget(),
        "cached DuckDB checkpoint saves took {:.3}ms",
        save_elapsed.as_secs_f64() * 1_000.0
    );
    assert!(
        load_elapsed <= cached_facade_budget(),
        "cached DuckDB checkpoint loads took {:.3}ms",
        load_elapsed.as_secs_f64() * 1_000.0
    );
}

#[tokio::test(flavor = "current_thread")]
async fn duckdb_checkpoint_store_reopened_cache_probe_reports_timing() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bundle = write_business_rule_bundle(&temp_dir);
    let package = ok_of(
        load_bpmn_package_from_files(&bundle.bpmn_path, std::slice::from_ref(&bundle.dmn_path)),
        "bundle should load from disk",
    );
    let database_path = temp_dir.path().join("reopened-cache-probe.duckdb");
    let writer_store = QianjiBpmnCheckpointStore::duckdb(database_path.clone());
    let checkpoints = (0..CACHED_DUCKDB_CHECKPOINT_PROBE_COUNT)
        .map(|index| {
            let sequence = usize_to_u64(index) + 1;
            let session = ok_of(
                QianjiBpmnSession::new(
                    Arc::clone(&package),
                    "review",
                    BpmnInstanceInit::new(
                        format!("wf_reopened_duckdb_{index:04}"),
                        json!({
                            "risk": if index % 2 == 0 { "low" } else { "high" },
                            "score": index,
                            "route": format!("lane_{}", index % 8),
                        }),
                        5 + sequence,
                    ),
                ),
                "session should be created",
            );
            session.checkpoint()
        })
        .collect::<Vec<_>>();

    for checkpoint in &checkpoints {
        ok_of(
            writer_store.save(checkpoint).await,
            "writer DuckDB checkpoint save should succeed",
        );
    }

    let reopened_store = QianjiBpmnCheckpointStore::duckdb(database_path);
    let load_started = Instant::now();
    for checkpoint in &checkpoints {
        let loaded = ok_of(
            reopened_store
                .load(checkpoint.state.instance_id.as_ref())
                .await,
            "reopened DuckDB checkpoint load should succeed",
        )
        .unwrap_or_else(|| panic!("reopened DuckDB checkpoint should exist"));
        assert_eq!(loaded.sequence, checkpoint.sequence);
    }
    let load_elapsed = load_started.elapsed();

    println!(
        "qianji duckdb checkpoint-store reopened cache perf: instances={} load_ms={:.3} load_avg_us={}",
        CACHED_DUCKDB_CHECKPOINT_PROBE_COUNT,
        load_elapsed.as_secs_f64() * 1_000.0,
        avg_us(load_elapsed, CACHED_DUCKDB_CHECKPOINT_PROBE_COUNT),
    );

    assert!(
        load_elapsed <= cached_facade_budget(),
        "reopened DuckDB checkpoint loads took {:.3}ms",
        load_elapsed.as_secs_f64() * 1_000.0
    );
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

fn cached_facade_budget() -> Duration {
    if std::env::var_os("NEXTEST_RUN_ID").is_some() {
        Duration::from_secs(20)
    } else if std::env::var_os("CI").is_some() {
        Duration::from_secs(10)
    } else {
        Duration::from_secs(5)
    }
}

fn avg_us(elapsed: Duration, count: usize) -> u128 {
    elapsed.as_micros() / usize_to_u128(count)
}

fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or_else(|error| panic!("usize should fit into u64: {error}"))
}

fn usize_to_u128(value: usize) -> u128 {
    u128::try_from(value).unwrap_or_else(|error| panic!("usize should fit into u128: {error}"))
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
