//! Criterion benchmarks for DB store persistence boundaries.

#[cfg(any(feature = "duckdb", feature = "qianji-bpmn-workflow-state"))]
use criterion::{BatchSize, Throughput};
use criterion::{Criterion, black_box};

#[cfg(feature = "duckdb")]
use std::sync::Arc;

#[cfg(feature = "duckdb")]
use xiuxian_db_store::duckdb::{DuckLakeRecordBatchAppender, DuckLakeTableRef};

#[cfg(feature = "duckdb")]
const DB_STORE_DUCKLAKE_BENCH_BATCHES: usize = 8;
#[cfg(feature = "duckdb")]
const DB_STORE_DUCKLAKE_BENCH_ROWS_PER_BATCH: usize = 128;

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

#[cfg(feature = "duckdb")]
fn bench_ducklake_arrow_appender(c: &mut Criterion) {
    let mut group = c.benchmark_group("db_store_ducklake_arrow_appender");
    group.throughput(Throughput::Elements(
        (DB_STORE_DUCKLAKE_BENCH_BATCHES * DB_STORE_DUCKLAKE_BENCH_ROWS_PER_BATCH) as u64,
    ));
    group.bench_function("reuse_open_appender_for_batches", |bench| {
        bench.iter_batched(
            build_ducklake_appender_fixture,
            |fixture| {
                let mut appender =
                    DuckLakeRecordBatchAppender::open(&fixture.connection, &fixture.table)
                        .unwrap_or_else(|error| panic!("open benchmark appender: {error}"));
                let row_total = appender
                    .append_batches(fixture.batches)
                    .unwrap_or_else(|error| panic!("append benchmark batches: {error}"));
                appender
                    .flush()
                    .unwrap_or_else(|error| panic!("flush benchmark appender: {error}"));
                black_box(row_total);
            },
            BatchSize::LargeInput,
        );
    });
    group.finish();
}

#[cfg(feature = "duckdb")]
struct DuckLakeAppenderFixture {
    connection: ::duckdb::Connection,
    table: DuckLakeTableRef,
    batches: Vec<::duckdb::arrow::record_batch::RecordBatch>,
}

#[cfg(feature = "duckdb")]
fn build_ducklake_appender_fixture() -> DuckLakeAppenderFixture {
    let connection = ::duckdb::Connection::open_in_memory()
        .unwrap_or_else(|error| panic!("open DuckDB: {error}"));
    connection
        .execute_batch(
            "CREATE TABLE events (tenant_id VARCHAR, case_id VARCHAR, event_type VARCHAR);",
        )
        .unwrap_or_else(|error| panic!("create benchmark table: {error}"));
    let table = DuckLakeTableRef::main_schema("memory", "events");
    let batches = (0..DB_STORE_DUCKLAKE_BENCH_BATCHES)
        .map(ducklake_appender_record_batch)
        .collect();
    DuckLakeAppenderFixture {
        connection,
        table,
        batches,
    }
}

#[cfg(feature = "duckdb")]
fn ducklake_appender_record_batch(index: usize) -> ::duckdb::arrow::record_batch::RecordBatch {
    let schema = Arc::new(::duckdb::arrow::datatypes::Schema::new(vec![
        ::duckdb::arrow::datatypes::Field::new(
            "tenant_id",
            ::duckdb::arrow::datatypes::DataType::Utf8,
            false,
        ),
        ::duckdb::arrow::datatypes::Field::new(
            "case_id",
            ::duckdb::arrow::datatypes::DataType::Utf8,
            false,
        ),
        ::duckdb::arrow::datatypes::Field::new(
            "event_type",
            ::duckdb::arrow::datatypes::DataType::Utf8,
            false,
        ),
    ]));
    let tenant_ids = vec!["tenant-a"; DB_STORE_DUCKLAKE_BENCH_ROWS_PER_BATCH];
    let case_ids = (0..DB_STORE_DUCKLAKE_BENCH_ROWS_PER_BATCH)
        .map(|row| {
            format!(
                "case-{}",
                (index * DB_STORE_DUCKLAKE_BENCH_ROWS_PER_BATCH + row) % 64
            )
        })
        .collect::<Vec<_>>();
    let event_types = (0..DB_STORE_DUCKLAKE_BENCH_ROWS_PER_BATCH)
        .map(|row| {
            if row % 3 == 0 {
                "tool.call"
            } else if row % 3 == 1 {
                "llm.call"
            } else {
                "bpmn.step"
            }
        })
        .collect::<Vec<_>>();

    ::duckdb::arrow::record_batch::RecordBatch::try_new(
        schema,
        vec![
            Arc::new(::duckdb::arrow::array::StringArray::from(tenant_ids))
                as ::duckdb::arrow::array::ArrayRef,
            Arc::new(::duckdb::arrow::array::StringArray::from(case_ids)),
            Arc::new(::duckdb::arrow::array::StringArray::from(event_types)),
        ],
    )
    .unwrap_or_else(|error| panic!("build benchmark Arrow batch: {error}"))
}

#[cfg(not(feature = "duckdb"))]
fn bench_ducklake_arrow_appender(c: &mut Criterion) {
    let mut group = c.benchmark_group("db_store_ducklake_arrow_appender");
    group.bench_function("feature_disabled", |bench| {
        bench.iter(|| black_box("enable duckdb for DuckLake appender benchmarks"));
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
    bench_ducklake_arrow_appender(&mut criterion);
    criterion.final_summary();
}
