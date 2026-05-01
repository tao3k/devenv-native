use super::support::{
    must_ok, must_some, open_file_store, open_store_path, sample_checkpoint_with_package,
    sample_package,
};
use crate::qianji_bpmn::QianjiBpmnDataStoreError;
use qianji_bpmn_engine::BpmnCheckpointEnvelope;
use serde_json::{Value, json};
use std::path::Path;
use std::time::{Duration, Instant};
use tempfile::TempDir;

const BENCH_SLACK_ENV: &str = "QIANJI_DUCKDB_WORKFLOW_STATE_BENCH_SLACK_FACTOR";
const DEFAULT_BENCH_SLACK_FACTOR: f64 = 2.0;
const REUSED_STORE_WORKFLOW_STATE_COUNT: usize = 1_000;
const OPEN_PER_OPERATION_WORKFLOW_STATE_COUNT: usize = 128;

#[derive(Debug, Clone, Copy)]
struct TimingSummary {
    count: usize,
    total: Duration,
    p50: Duration,
    p95: Duration,
    max: Duration,
}

impl TimingSummary {
    fn from_samples(samples: Vec<Duration>) -> Self {
        assert!(!samples.is_empty(), "timing summary requires samples");
        let total = samples
            .iter()
            .copied()
            .fold(Duration::ZERO, |acc, elapsed| acc + elapsed);
        let mut sorted = samples;
        sorted.sort_unstable();
        Self {
            count: sorted.len(),
            total,
            p50: percentile(&sorted, 50),
            p95: percentile(&sorted, 95),
            max: percentile(&sorted, 100),
        }
    }

    fn total_ms(self) -> f64 {
        self.total.as_secs_f64() * 1_000.0
    }

    fn p50_us(self) -> u128 {
        self.p50.as_micros()
    }

    fn p95_us(self) -> u128 {
        self.p95.as_micros()
    }

    fn max_us(self) -> u128 {
        self.max.as_micros()
    }

    fn avg_us(self) -> u128 {
        self.total.as_micros() / must_u128(self.count)
    }
}

#[test]
fn duckdb_workflow_state_codec_probe_reports_timing() {
    let checkpoints = sample_checkpoints("wf_duckdb_codec", REUSED_STORE_WORKFLOW_STATE_COUNT, 1);
    let mut encoded_payloads = Vec::with_capacity(checkpoints.len());
    let encode = measure_store_ops(REUSED_STORE_WORKFLOW_STATE_COUNT, |index| {
        let payload = serde_json::to_string(&checkpoints[index]).map_err(|error| {
            QianjiBpmnDataStoreError::Codec {
                operation: "encode_checkpoint_probe",
                message: error.to_string(),
            }
        })?;
        encoded_payloads.push(payload);
        Ok(())
    });
    let total_bytes: usize = encoded_payloads.iter().map(String::len).sum();
    let decode =
        measure_store_ops(REUSED_STORE_WORKFLOW_STATE_COUNT, |index| {
            let _: BpmnCheckpointEnvelope = serde_json::from_str(&encoded_payloads[index])
                .map_err(|error| QianjiBpmnDataStoreError::Codec {
                    operation: "decode_checkpoint_probe",
                    message: error.to_string(),
                })?;
            Ok(())
        });

    println!(
        "qianji duckdb workflow-state codec perf: instances={} payload_profile={} encode_ms={:.3} decode_ms={:.3} encode_avg_us={} decode_avg_us={} avg_bytes={} total_bytes={}",
        REUSED_STORE_WORKFLOW_STATE_COUNT,
        payload_profile_label(),
        encode.total_ms(),
        decode.total_ms(),
        encode.avg_us(),
        decode.avg_us(),
        total_bytes / REUSED_STORE_WORKFLOW_STATE_COUNT,
        total_bytes,
    );

    assert_within_budget(
        "workflow-state checkpoint encode",
        encode.total,
        benchmark_budget(
            Duration::from_secs(4),
            Duration::from_secs(8),
            Duration::from_secs(20),
        ),
    );
    assert_within_budget(
        "workflow-state checkpoint decode",
        decode.total,
        benchmark_budget(
            Duration::from_secs(4),
            Duration::from_secs(8),
            Duration::from_secs(20),
        ),
    );
}

#[test]
fn duckdb_workflow_state_append_log_probe_reports_timing() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
    let store = open_file_store(&temp_dir, "workflow-state-append-log-perf.duckdb");
    let first = sample_checkpoints("wf_duckdb_append_log", REUSED_STORE_WORKFLOW_STATE_COUNT, 1);
    let updated = sample_checkpoints("wf_duckdb_append_log", REUSED_STORE_WORKFLOW_STATE_COUNT, 2);

    let append = measure_batch_op(REUSED_STORE_WORKFLOW_STATE_COUNT, || {
        store
            .append_workflow_state_snapshots(first.iter())
            .map(|written| assert_eq!(written, REUSED_STORE_WORKFLOW_STATE_COUNT))
    });
    let append_update = measure_batch_op(REUSED_STORE_WORKFLOW_STATE_COUNT, || {
        store
            .append_workflow_state_snapshots(updated.iter())
            .map(|written| assert_eq!(written, REUSED_STORE_WORKFLOW_STATE_COUNT))
    });
    let load_latest = measure_store_ops(REUSED_STORE_WORKFLOW_STATE_COUNT, |index| {
        let instance_id = first[index].state.instance_id.as_ref();
        let loaded = must_some(
            store.load_latest_workflow_state_snapshot(instance_id)?,
            "latest workflow-state append-log sample should exist",
        );
        assert_eq!(loaded.sequence, 2);
        Ok(())
    });

    println!(
        "qianji duckdb workflow-state append-log perf: instances={} payload_profile={} append_arrow_ms={:.3} append_update_arrow_ms={:.3} load_latest_ms={:.3} append_avg_us={} append_update_avg_us={} load_latest_avg_us={} load_latest_p95_us={}",
        REUSED_STORE_WORKFLOW_STATE_COUNT,
        payload_profile_label(),
        append.total_ms(),
        append_update.total_ms(),
        load_latest.total_ms(),
        append.avg_us(),
        append_update.avg_us(),
        load_latest.avg_us(),
        load_latest.p95_us(),
    );

    assert_within_budget(
        "append-log workflow-state append",
        append.total,
        benchmark_budget(
            Duration::from_secs(4),
            Duration::from_secs(8),
            Duration::from_secs(20),
        ),
    );
    assert_within_budget(
        "append-log workflow-state append update",
        append_update.total,
        benchmark_budget(
            Duration::from_secs(4),
            Duration::from_secs(8),
            Duration::from_secs(20),
        ),
    );
    assert_within_budget(
        "append-log workflow-state latest load",
        load_latest.total,
        benchmark_budget(
            Duration::from_secs(8),
            Duration::from_secs(16),
            Duration::from_secs(40),
        ),
    );
}

#[test]
fn duckdb_workflow_state_compacted_latest_probe_reports_timing() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
    let store = open_file_store(&temp_dir, "workflow-state-compacted-latest-perf.duckdb");
    let first = sample_checkpoints(
        "wf_duckdb_compacted_latest",
        REUSED_STORE_WORKFLOW_STATE_COUNT,
        1,
    );
    let updated = sample_checkpoints(
        "wf_duckdb_compacted_latest",
        REUSED_STORE_WORKFLOW_STATE_COUNT,
        2,
    );

    let append = measure_batch_op(REUSED_STORE_WORKFLOW_STATE_COUNT, || {
        store
            .append_workflow_state_snapshots(first.iter())
            .map(|written| assert_eq!(written, REUSED_STORE_WORKFLOW_STATE_COUNT))
    });
    let append_update = measure_batch_op(REUSED_STORE_WORKFLOW_STATE_COUNT, || {
        store
            .append_workflow_state_snapshots(updated.iter())
            .map(|written| assert_eq!(written, REUSED_STORE_WORKFLOW_STATE_COUNT))
    });
    let compact = measure_batch_op(REUSED_STORE_WORKFLOW_STATE_COUNT, || {
        store.compact_workflow_state_latest_snapshots()
    });
    let load_all_compacted = measure_batch_op(REUSED_STORE_WORKFLOW_STATE_COUNT, || {
        store
            .load_compacted_workflow_state_snapshots()
            .map(|loaded| assert_eq!(loaded.len(), REUSED_STORE_WORKFLOW_STATE_COUNT))
    });
    let load_compacted = measure_store_ops(REUSED_STORE_WORKFLOW_STATE_COUNT, |index| {
        let instance_id = first[index].state.instance_id.as_ref();
        let loaded = must_some(
            store.load_compacted_workflow_state_snapshot(instance_id)?,
            "compacted latest workflow-state sample should exist",
        );
        assert_eq!(loaded.sequence, 2);
        Ok(())
    });

    println!(
        "qianji duckdb workflow-state compacted-latest perf: instances={} payload_profile={} append_arrow_ms={:.3} append_update_arrow_ms={:.3} compact_ms={:.3} hydrate_all_ms={:.3} load_compacted_ms={:.3} compact_avg_us={} hydrate_all_avg_us={} load_compacted_avg_us={} load_compacted_p95_us={}",
        REUSED_STORE_WORKFLOW_STATE_COUNT,
        payload_profile_label(),
        append.total_ms(),
        append_update.total_ms(),
        compact.total_ms(),
        load_all_compacted.total_ms(),
        load_compacted.total_ms(),
        compact.avg_us(),
        load_all_compacted.avg_us(),
        load_compacted.avg_us(),
        load_compacted.p95_us(),
    );

    assert_within_budget(
        "compacted-latest workflow-state append",
        append.total,
        benchmark_budget(
            Duration::from_secs(4),
            Duration::from_secs(8),
            Duration::from_secs(20),
        ),
    );
    assert_within_budget(
        "compacted-latest workflow-state append update",
        append_update.total,
        benchmark_budget(
            Duration::from_secs(4),
            Duration::from_secs(8),
            Duration::from_secs(20),
        ),
    );
    assert_within_budget(
        "compacted-latest workflow-state compact",
        compact.total,
        benchmark_budget(
            Duration::from_secs(4),
            Duration::from_secs(8),
            Duration::from_secs(20),
        ),
    );
    assert_within_budget(
        "compacted-latest workflow-state load",
        load_compacted.total,
        benchmark_budget(
            Duration::from_secs(8),
            Duration::from_secs(16),
            Duration::from_secs(40),
        ),
    );
    assert_within_budget(
        "compacted-latest workflow-state hydrate all",
        load_all_compacted.total,
        benchmark_budget(
            Duration::from_secs(4),
            Duration::from_secs(8),
            Duration::from_secs(20),
        ),
    );
}

#[test]
fn duckdb_workflow_state_point_append_probe_reports_timing() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
    let store = open_file_store(&temp_dir, "workflow-state-point-append-perf.duckdb");
    let first = sample_checkpoints(
        "wf_duckdb_point_append",
        REUSED_STORE_WORKFLOW_STATE_COUNT,
        1,
    );
    let updated = sample_checkpoints(
        "wf_duckdb_point_append",
        REUSED_STORE_WORKFLOW_STATE_COUNT,
        2,
    );

    let append = measure_store_ops(REUSED_STORE_WORKFLOW_STATE_COUNT, |index| {
        store.append_workflow_state_snapshot(&first[index])
    });
    let append_update = measure_store_ops(REUSED_STORE_WORKFLOW_STATE_COUNT, |index| {
        store.append_workflow_state_snapshot(&updated[index])
    });
    let load_latest = measure_store_ops(REUSED_STORE_WORKFLOW_STATE_COUNT, |index| {
        let instance_id = first[index].state.instance_id.as_ref();
        let loaded = must_some(
            store.load_latest_workflow_state_snapshot(instance_id)?,
            "point-append workflow-state sample should exist",
        );
        assert_eq!(loaded.sequence, 2);
        Ok(())
    });

    println!(
        "qianji duckdb workflow-state point-append perf: instances={} payload_profile={} append_ms={:.3} append_update_ms={:.3} load_latest_ms={:.3} append_avg_us={} append_update_avg_us={} load_latest_avg_us={} load_latest_p95_us={}",
        REUSED_STORE_WORKFLOW_STATE_COUNT,
        payload_profile_label(),
        append.total_ms(),
        append_update.total_ms(),
        load_latest.total_ms(),
        append.avg_us(),
        append_update.avg_us(),
        load_latest.avg_us(),
        load_latest.p95_us(),
    );

    assert_within_budget(
        "point-append workflow-state append",
        append.total,
        benchmark_budget(
            Duration::from_secs(8),
            Duration::from_secs(16),
            Duration::from_secs(40),
        ),
    );
    assert_within_budget(
        "point-append workflow-state append update",
        append_update.total,
        benchmark_budget(
            Duration::from_secs(8),
            Duration::from_secs(16),
            Duration::from_secs(40),
        ),
    );
    assert_within_budget(
        "point-append workflow-state latest load",
        load_latest.total,
        benchmark_budget(
            Duration::from_secs(8),
            Duration::from_secs(16),
            Duration::from_secs(40),
        ),
    );
}

#[test]
fn duckdb_workflow_state_reused_store_probe_reports_timing() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
    let store = open_file_store(&temp_dir, "workflow-state-reused-perf.duckdb");
    let first = sample_checkpoints("wf_duckdb_reused", REUSED_STORE_WORKFLOW_STATE_COUNT, 1);
    let updated = sample_checkpoints("wf_duckdb_reused", REUSED_STORE_WORKFLOW_STATE_COUNT, 2);

    let upsert = measure_batch_op(REUSED_STORE_WORKFLOW_STATE_COUNT, || {
        store
            .upsert_workflow_states(first.iter())
            .map(|written| assert_eq!(written, REUSED_STORE_WORKFLOW_STATE_COUNT))
    });
    let load = measure_store_ops(REUSED_STORE_WORKFLOW_STATE_COUNT, |index| {
        let instance_id = first[index].state.instance_id.as_ref();
        let loaded = must_some(
            store.load_workflow_state(instance_id)?,
            "workflow-state sample should exist after bulk upsert",
        );
        assert_eq!(loaded.sequence, 1);
        Ok(())
    });
    let overwrite = measure_batch_op(REUSED_STORE_WORKFLOW_STATE_COUNT, || {
        store
            .upsert_workflow_states(updated.iter())
            .map(|written| assert_eq!(written, REUSED_STORE_WORKFLOW_STATE_COUNT))
    });
    let delete = measure_batch_op(REUSED_STORE_WORKFLOW_STATE_COUNT, || {
        let instance_ids = first
            .iter()
            .map(|checkpoint| checkpoint.state.instance_id.as_ref());
        store
            .delete_workflow_states(instance_ids)
            .map(|deleted| assert_eq!(deleted, REUSED_STORE_WORKFLOW_STATE_COUNT))
    });

    println!(
        "qianji duckdb workflow-state reused-store perf: instances={} payload_profile={} upsert_batch_ms={:.3} load_ms={:.3} overwrite_batch_ms={:.3} delete_batch_ms={:.3} upsert_avg_us={} load_avg_us={} overwrite_avg_us={} delete_avg_us={} load_p95_us={}",
        REUSED_STORE_WORKFLOW_STATE_COUNT,
        payload_profile_label(),
        upsert.total_ms(),
        load.total_ms(),
        overwrite.total_ms(),
        delete.total_ms(),
        upsert.avg_us(),
        load.avg_us(),
        overwrite.avg_us(),
        delete.avg_us(),
        load.p95_us(),
    );

    assert_within_budget(
        "reused-store workflow-state upsert",
        upsert.total,
        benchmark_budget(
            Duration::from_secs(8),
            Duration::from_secs(16),
            Duration::from_secs(40),
        ),
    );
    assert_within_budget(
        "reused-store workflow-state load",
        load.total,
        benchmark_budget(
            Duration::from_secs(8),
            Duration::from_secs(16),
            Duration::from_secs(40),
        ),
    );
    assert_within_budget(
        "reused-store workflow-state overwrite",
        overwrite.total,
        benchmark_budget(
            Duration::from_secs(8),
            Duration::from_secs(16),
            Duration::from_secs(40),
        ),
    );
    assert_within_budget(
        "reused-store workflow-state delete",
        delete.total,
        benchmark_budget(
            Duration::from_secs(8),
            Duration::from_secs(16),
            Duration::from_secs(40),
        ),
    );
}

#[test]
fn duckdb_workflow_state_latest_table_probe_reports_timing() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
    let store = open_file_store(&temp_dir, "workflow-state-latest-table-perf.duckdb");
    let first = sample_checkpoints(
        "wf_duckdb_latest_table",
        REUSED_STORE_WORKFLOW_STATE_COUNT,
        1,
    );
    let updated = sample_checkpoints(
        "wf_duckdb_latest_table",
        REUSED_STORE_WORKFLOW_STATE_COUNT,
        2,
    );

    let upsert = measure_store_ops(REUSED_STORE_WORKFLOW_STATE_COUNT, |index| {
        store.upsert_latest_workflow_state_snapshot(&first[index])
    });
    let overwrite = measure_store_ops(REUSED_STORE_WORKFLOW_STATE_COUNT, |index| {
        store.upsert_latest_workflow_state_snapshot(&updated[index])
    });
    let load_latest = measure_store_ops(REUSED_STORE_WORKFLOW_STATE_COUNT, |index| {
        let instance_id = first[index].state.instance_id.as_ref();
        let loaded = must_some(
            store.load_latest_workflow_state_snapshot(instance_id)?,
            "latest-table workflow-state sample should exist",
        );
        assert_eq!(loaded.sequence, 2);
        Ok(())
    });

    println!(
        "qianji duckdb workflow-state latest-table perf: instances={} payload_profile={} upsert_ms={:.3} overwrite_ms={:.3} load_latest_ms={:.3} upsert_avg_us={} overwrite_avg_us={} load_latest_avg_us={} load_latest_p95_us={}",
        REUSED_STORE_WORKFLOW_STATE_COUNT,
        payload_profile_label(),
        upsert.total_ms(),
        overwrite.total_ms(),
        load_latest.total_ms(),
        upsert.avg_us(),
        overwrite.avg_us(),
        load_latest.avg_us(),
        load_latest.p95_us(),
    );

    assert_within_budget(
        "latest-table workflow-state upsert",
        upsert.total,
        benchmark_budget(
            Duration::from_secs(8),
            Duration::from_secs(16),
            Duration::from_secs(40),
        ),
    );
    assert_within_budget(
        "latest-table workflow-state overwrite",
        overwrite.total,
        benchmark_budget(
            Duration::from_secs(8),
            Duration::from_secs(16),
            Duration::from_secs(40),
        ),
    );
    assert_within_budget(
        "latest-table workflow-state latest load",
        load_latest.total,
        benchmark_budget(
            Duration::from_secs(8),
            Duration::from_secs(16),
            Duration::from_secs(40),
        ),
    );
}

#[test]
fn duckdb_workflow_state_open_per_operation_probe_reports_timing() {
    let temp_dir = must_ok(TempDir::new(), "temp dir should allocate");
    let database_path = temp_dir
        .path()
        .join("workflow-state-open-per-op-perf.duckdb");
    let checkpoints = sample_checkpoints(
        "wf_duckdb_open_per_op",
        OPEN_PER_OPERATION_WORKFLOW_STATE_COUNT,
        1,
    );

    let save = measure_open_per_operation_ops(
        &database_path,
        OPEN_PER_OPERATION_WORKFLOW_STATE_COUNT,
        |store, index| store.upsert_workflow_state(&checkpoints[index]),
    );
    let load = measure_open_per_operation_ops(
        &database_path,
        OPEN_PER_OPERATION_WORKFLOW_STATE_COUNT,
        |store, index| {
            let checkpoint = &checkpoints[index];
            let instance_id = checkpoint.state.instance_id.as_ref();
            let loaded = must_some(
                store.load_workflow_state(instance_id)?,
                "workflow-state sample should exist after open-per-operation save",
            );
            assert_eq!(loaded.sequence, checkpoint.sequence);
            Ok(())
        },
    );

    println!(
        "qianji duckdb workflow-state open-per-operation perf: instances={} payload_profile={} save_ms={:.3} load_ms={:.3} save_avg_us={} load_avg_us={} save_p50_us={} load_p50_us={} save_p95_us={} load_p95_us={} save_max_us={} load_max_us={}",
        OPEN_PER_OPERATION_WORKFLOW_STATE_COUNT,
        payload_profile_label(),
        save.total_ms(),
        load.total_ms(),
        save.avg_us(),
        load.avg_us(),
        save.p50_us(),
        load.p50_us(),
        save.p95_us(),
        load.p95_us(),
        save.max_us(),
        load.max_us(),
    );

    assert_within_budget(
        "open-per-operation workflow-state save",
        save.total,
        benchmark_budget(
            Duration::from_secs(20),
            Duration::from_secs(40),
            Duration::from_secs(90),
        ),
    );
    assert_within_budget(
        "open-per-operation workflow-state load",
        load.total,
        benchmark_budget(
            Duration::from_secs(20),
            Duration::from_secs(40),
            Duration::from_secs(90),
        ),
    );
}

fn sample_checkpoints(
    instance_prefix: &str,
    count: usize,
    sequence: u64,
) -> Vec<BpmnCheckpointEnvelope> {
    let package = sample_package();
    (0..count)
        .map(|index| {
            sample_checkpoint_with_package(
                &package,
                &format!("{instance_prefix}_{index:05}"),
                sequence,
                sample_variables(index, sequence),
            )
        })
        .collect()
}

fn sample_variables(index: usize, sequence: u64) -> Value {
    json!({
        "approved": index.is_multiple_of(2),
        "amount": index.saturating_mul(17),
        "sequence": sequence,
        "decision_context": {
            "risk_band": if index.is_multiple_of(3) { "high" } else { "low" },
            "route": format!("lane_{}", index % 16),
            "payload_profile": payload_profile_label(),
        },
        "host_results": [
            { "node_id": "service_a", "status": "completed", "score": index % 97 },
            { "node_id": "business_rule", "status": "completed", "decision": index % 5 },
            { "node_id": "user_review", "status": "waiting", "resume_key": format!("resume_{index:05}") },
        ],
    })
}

fn payload_profile_label() -> &'static str {
    "medium-json-checkpoint"
}

fn measure_store_ops<F>(count: usize, mut operation: F) -> TimingSummary
where
    F: FnMut(usize) -> Result<(), QianjiBpmnDataStoreError>,
{
    let mut samples = Vec::with_capacity(count);
    for index in 0..count {
        let started = Instant::now();
        must_ok(
            operation(index),
            "DuckDB workflow-state operation should succeed",
        );
        samples.push(started.elapsed());
    }
    TimingSummary::from_samples(samples)
}

fn measure_batch_op<F>(count: usize, mut operation: F) -> TimingSummary
where
    F: FnMut() -> Result<(), QianjiBpmnDataStoreError>,
{
    let started = Instant::now();
    must_ok(
        operation(),
        "DuckDB workflow-state batch operation should succeed",
    );
    TimingSummary {
        count,
        total: started.elapsed(),
        p50: Duration::ZERO,
        p95: Duration::ZERO,
        max: Duration::ZERO,
    }
}

fn measure_open_per_operation_ops<F>(
    database_path: &Path,
    count: usize,
    mut operation: F,
) -> TimingSummary
where
    F: FnMut(
        &crate::qianji_bpmn::QianjiBpmnDuckDbDataStore,
        usize,
    ) -> Result<(), QianjiBpmnDataStoreError>,
{
    let mut samples = Vec::with_capacity(count);
    for index in 0..count {
        let started = Instant::now();
        let store = open_store_path(database_path);
        must_ok(
            operation(&store, index),
            "DuckDB open-per-operation workflow-state operation should succeed",
        );
        samples.push(started.elapsed());
    }
    TimingSummary::from_samples(samples)
}

fn percentile(sorted: &[Duration], percentile: usize) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let index = sorted
        .len()
        .saturating_sub(1)
        .saturating_mul(percentile.min(100))
        / 100;
    sorted[index]
}

fn benchmark_budget(local: Duration, ci: Duration, nextest: Duration) -> Duration {
    let baseline = if std::env::var_os("NEXTEST_RUN_ID").is_some() {
        nextest
    } else if std::env::var_os("CI").is_some() {
        ci
    } else {
        local
    };
    baseline.mul_f64(benchmark_slack_factor())
}

fn benchmark_slack_factor() -> f64 {
    std::env::var(BENCH_SLACK_ENV)
        .ok()
        .and_then(|raw| raw.parse::<f64>().ok())
        .filter(|factor| factor.is_finite() && *factor >= 1.0)
        .unwrap_or(DEFAULT_BENCH_SLACK_FACTOR)
}

fn assert_within_budget(label: &str, elapsed: Duration, budget: Duration) {
    assert!(
        elapsed <= budget,
        "{label} took {:.3}ms, over budget {:.3}ms; set {BENCH_SLACK_ENV} to widen on slow hosts",
        elapsed.as_secs_f64() * 1_000.0,
        budget.as_secs_f64() * 1_000.0,
    );
}

fn must_u128(value: usize) -> u128 {
    u128::try_from(value).unwrap_or_else(|error| panic!("usize should fit into u128: {error}"))
}
