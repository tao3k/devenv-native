use std::collections::HashMap;
use std::sync::Arc;

use serial_test::file_serial;
use tempfile::TempDir;
use xiuxian_testing::{PerfBudget, PerfReport, PerfRunConfig, assert_perf_budget, run_sync_budget};
use xiuxian_vector::{
    LanceDataType, LanceField, LanceRecordBatch, LanceSchema, LanceStringArray, LanceUInt64Array,
    SearchEngineContext, write_lance_batches_to_parquet_file,
};
use xiuxian_wendao::duckdb::{
    DataFusionParquetQueryEngine, DuckDbDatabasePath, ParquetQueryEngine, SearchDuckDbRuntimeConfig,
};

use super::support::{env_f64, env_u64, env_usize};

const SUITE: &str = "xiuxian-wendao/perf";
const DATAFUSION_CASE: &str = "parquet_query_engine_datafusion_p95";
const DUCKDB_CASE: &str = "parquet_query_engine_duckdb_p95";
const TABLE_NAME: &str = "bench_docs";
const FIXTURE_ROWS: usize = 48_000;
const EXPECTED_RESULT_ROWS: usize = 32;
const SQL: &str = "SELECT doc_id, symbol, line_no \
FROM bench_docs \
WHERE language = 'rust' \
  AND kind = 'function' \
  AND symbol LIKE 'Symbol 00%' \
ORDER BY line_no DESC \
LIMIT 32";

#[test]
#[file_serial(wendao_perf_gate)]
fn parquet_query_engine_duckdb_vs_datafusion_p95_gate() -> Result<(), String> {
    let fixture = build_parquet_fixture().map_err(|error| error.to_string())?;
    let datafusion_engine = ParquetQueryEngine::DataFusion(DataFusionParquetQueryEngine::new(
        SearchEngineContext::new(),
    ));
    let duckdb_engine =
        ParquetQueryEngine::duckdb_from_runtime(duckdb_runtime(fixture.root_path()))
            .map_err(|error| error.to_string())?;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("failed to build parquet perf runtime: {error}"))?;

    runtime
        .block_on(
            datafusion_engine.ensure_parquet_table_registered(TABLE_NAME, fixture.parquet_path()),
        )
        .map_err(|error| format!("failed to register DataFusion parquet table: {error}"))?;
    runtime
        .block_on(duckdb_engine.ensure_parquet_table_registered(TABLE_NAME, fixture.parquet_path()))
        .map_err(|error| format!("failed to register DuckDB parquet table: {error}"))?;

    let config = parquet_perf_config();

    let mut datafusion_report =
        run_sync_budget(SUITE, DATAFUSION_CASE, &config, || -> Result<(), String> {
            query_engine_once(&runtime, &datafusion_engine)
        });
    datafusion_report.add_metadata("engine", "datafusion");
    datafusion_report.add_metadata("fixture_rows", FIXTURE_ROWS.to_string());
    datafusion_report.add_metadata("query", SQL);

    let mut duckdb_report =
        run_sync_budget(SUITE, DUCKDB_CASE, &config, || -> Result<(), String> {
            query_engine_once(&runtime, &duckdb_engine)
        });
    duckdb_report.add_metadata("engine", "duckdb");
    duckdb_report.add_metadata("fixture_rows", FIXTURE_ROWS.to_string());
    duckdb_report.add_metadata("query", SQL);

    let error_budget = PerfBudget {
        max_error_rate: Some(0.0),
        ..PerfBudget::new()
    };
    assert_perf_budget(&datafusion_report, &error_budget);
    assert_perf_budget(&duckdb_report, &error_budget);
    assert_duckdb_p95_ratio(&datafusion_report, &duckdb_report);

    println!(
        "parquet_query_engine_perf_gate: datafusion_p95_ms={:.3}, duckdb_p95_ms={:.3}, ratio={:.3}, datafusion_report={:?}, duckdb_report={:?}",
        datafusion_report.quantiles.p95_ms,
        duckdb_report.quantiles.p95_ms,
        p95_ratio(&datafusion_report, &duckdb_report),
        datafusion_report.report_path,
        duckdb_report.report_path
    );
    Ok(())
}

fn parquet_perf_config() -> PerfRunConfig {
    PerfRunConfig {
        warmup_samples: env_usize("XIUXIAN_WENDAO_PERF_PARQUET_QUERY_WARMUP", 2),
        samples: env_usize("XIUXIAN_WENDAO_PERF_PARQUET_QUERY_SAMPLES", 10),
        timeout_ms: env_u64("XIUXIAN_WENDAO_PERF_PARQUET_QUERY_TIMEOUT_MS", 1_500),
        concurrency: 1,
    }
}

fn duckdb_runtime(root: &std::path::Path) -> SearchDuckDbRuntimeConfig {
    SearchDuckDbRuntimeConfig {
        enabled: true,
        database_path: DuckDbDatabasePath::InMemory,
        temp_directory: root.join(".cache/duckdb-perf/tmp"),
        threads: 2,
        materialize_threshold_rows: 4_096,
        prefer_virtual_arrow: true,
    }
}

fn query_engine_once(
    runtime: &tokio::runtime::Runtime,
    engine: &ParquetQueryEngine,
) -> Result<(), String> {
    let batches = runtime
        .block_on(engine.query_batches(SQL))
        .map_err(|error| format!("failed to execute parquet perf query: {error}"))?;
    let row_count = batches
        .iter()
        .map(xiuxian_vector::EngineRecordBatch::num_rows)
        .sum::<usize>();
    if row_count != EXPECTED_RESULT_ROWS {
        return Err(format!(
            "expected {EXPECTED_RESULT_ROWS} parquet perf rows, got {row_count}"
        ));
    }
    Ok(())
}

fn assert_duckdb_p95_ratio(datafusion: &PerfReport, duckdb: &PerfReport) {
    let ratio = p95_ratio(datafusion, duckdb);
    let max_ratio = env_f64(
        "XIUXIAN_WENDAO_PERF_PARQUET_QUERY_MAX_P95_RATIO",
        if std::env::var_os("CI").is_some() {
            2.0
        } else {
            1.6
        },
    );
    assert!(
        ratio <= max_ratio,
        "DuckDB parquet query p95 ratio exceeded budget: duckdb_p95_ms={:.3}, datafusion_p95_ms={:.3}, ratio={:.3}, budget<={:.3}, duckdb_report={:?}, datafusion_report={:?}",
        duckdb.quantiles.p95_ms,
        datafusion.quantiles.p95_ms,
        ratio,
        max_ratio,
        duckdb.report_path,
        datafusion.report_path
    );
}

fn p95_ratio(datafusion: &PerfReport, duckdb: &PerfReport) -> f64 {
    let baseline = datafusion.quantiles.p95_ms.max(0.001);
    duckdb.quantiles.p95_ms / baseline
}

struct ParquetFixture {
    temp_dir: TempDir,
    parquet_path: std::path::PathBuf,
}

impl ParquetFixture {
    fn root_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    fn parquet_path(&self) -> &std::path::Path {
        self.parquet_path.as_path()
    }
}

fn build_parquet_fixture() -> Result<ParquetFixture, Box<dyn std::error::Error>> {
    let temp_dir = tempfile::Builder::new()
        .prefix("xiuxian_wendao_parquet_perf_")
        .tempdir()?;
    let parquet_path = temp_dir.path().join("bench_docs.parquet");
    let schema = Arc::new(LanceSchema::new_with_metadata(
        vec![
            LanceField::new("doc_id", LanceDataType::Utf8, false),
            LanceField::new("language", LanceDataType::Utf8, false),
            LanceField::new("kind", LanceDataType::Utf8, false),
            LanceField::new("symbol", LanceDataType::Utf8, false),
            LanceField::new("line_no", LanceDataType::UInt64, false),
        ],
        HashMap::from([("domain".to_string(), TABLE_NAME.to_string())]),
    ));

    let mut doc_ids = Vec::with_capacity(FIXTURE_ROWS);
    let mut languages = Vec::with_capacity(FIXTURE_ROWS);
    let mut kinds = Vec::with_capacity(FIXTURE_ROWS);
    let mut symbols = Vec::with_capacity(FIXTURE_ROWS);
    let mut line_numbers = Vec::with_capacity(FIXTURE_ROWS);

    for index in 0..FIXTURE_ROWS {
        doc_ids.push(format!("doc-{index:05}"));
        languages.push(match index % 3 {
            0 => "rust".to_string(),
            1 => "python".to_string(),
            _ => "markdown".to_string(),
        });
        kinds.push(match index % 5 {
            0 => "function".to_string(),
            1 => "struct".to_string(),
            2 => "enum".to_string(),
            3 => "trait".to_string(),
            _ => "module".to_string(),
        });
        symbols.push(format!("Symbol {index:05}"));
        line_numbers.push(u64::try_from(index % 8_192).unwrap_or(u64::MAX));
    }

    let batch = LanceRecordBatch::try_new(
        schema,
        vec![
            Arc::new(LanceStringArray::from(doc_ids)),
            Arc::new(LanceStringArray::from(languages)),
            Arc::new(LanceStringArray::from(kinds)),
            Arc::new(LanceStringArray::from(symbols)),
            Arc::new(LanceUInt64Array::from(line_numbers)),
        ],
    )?;
    write_lance_batches_to_parquet_file(&parquet_path, &[batch])?;

    Ok(ParquetFixture {
        temp_dir,
        parquet_path,
    })
}
