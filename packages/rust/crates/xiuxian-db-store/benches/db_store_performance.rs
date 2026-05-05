//! Criterion benchmarks for DB store persistence boundaries.

use criterion::{BatchSize, Criterion, Throughput, black_box};

#[cfg(feature = "qianji-bpmn-workflow-state")]
use serde_json::json;
#[cfg(feature = "qianji-bpmn-workflow-state")]
use tempfile::TempDir;
#[cfg(feature = "qianji-bpmn-workflow-state")]
use xiuxian_db_store::qianji_bpmn::{
    QianjiBpmnDataRecord, QianjiBpmnDuckDbDataStore, QianjiBpmnDuckDbDataStoreConfig,
};

#[cfg(feature = "qianji-bpmn-workflow-state")]
const DB_STORE_BENCH_RECORDS: usize = 64;

#[cfg(feature = "qianji-bpmn-workflow-state")]
fn bench_qianji_bpmn_duckdb_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("db_store_qianji_bpmn_duckdb");
    group.throughput(Throughput::Elements(DB_STORE_BENCH_RECORDS as u64));
    group.bench_function("upsert_and_load_records", |bench| {
        bench.iter_batched(
            build_qianji_bpmn_fixture,
            |fixture| {
                for record in &fixture.records {
                    fixture
                        .store
                        .upsert_record(record)
                        .unwrap_or_else(|error| panic!("upsert benchmark record: {error}"));
                }
                for record in &fixture.records {
                    let loaded = fixture
                        .store
                        .load_record(&record.instance_id, &record.record_key)
                        .unwrap_or_else(|error| panic!("load benchmark record: {error}"))
                        .unwrap_or_else(|| panic!("benchmark record should exist"));
                    black_box(loaded);
                }
            },
            BatchSize::LargeInput,
        );
    });
    group.finish();
}

#[cfg(feature = "qianji-bpmn-workflow-state")]
struct QianjiBpmnStoreFixture {
    _temp_dir: TempDir,
    store: QianjiBpmnDuckDbDataStore,
    records: Vec<QianjiBpmnDataRecord>,
}

#[cfg(feature = "qianji-bpmn-workflow-state")]
fn build_qianji_bpmn_fixture() -> QianjiBpmnStoreFixture {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("create tempdir: {error}"));
    let store = QianjiBpmnDuckDbDataStore::open(QianjiBpmnDuckDbDataStoreConfig::file(
        temp_dir.path().join("workflow-state.duckdb"),
    ))
    .unwrap_or_else(|error| panic!("open benchmark store: {error}"));
    let records = (0..DB_STORE_BENCH_RECORDS)
        .map(|index| {
            QianjiBpmnDataRecord::new(
                format!("bench-instance-{}", index % 8),
                format!("record-{index}"),
                json!({
                    "index": index,
                    "status": "waiting",
                    "payload": format!("workflow-state-payload-{index}"),
                }),
                1_700_000_000_000 + index as u64,
            )
        })
        .collect();
    QianjiBpmnStoreFixture {
        _temp_dir: temp_dir,
        store,
        records,
    }
}

#[cfg(not(feature = "qianji-bpmn-workflow-state"))]
fn bench_qianji_bpmn_duckdb_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("db_store_qianji_bpmn_duckdb");
    group.bench_function("feature_disabled", |bench| {
        bench.iter(|| black_box("enable qianji-bpmn-workflow-state for storage benchmarks"));
    });
    group.finish();
}

fn main() {
    let mut criterion = Criterion::default().configure_from_args();
    bench_qianji_bpmn_duckdb_store(&mut criterion);
    criterion.final_summary();
}
