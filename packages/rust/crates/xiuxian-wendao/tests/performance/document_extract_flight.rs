use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use arrow::array::{Array, StringArray};
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;
use arrow_flight::FlightDescriptor;
use arrow_flight::client::FlightClient;
use arrow_flight::flight_service_client::FlightServiceClient as TonicFlightServiceClient;
use futures::TryStreamExt;
use futures::future::try_join_all;
use serde::Serialize;
use tonic::transport::{Channel, Endpoint};
use xiuxian_wendao_runtime::transport::{
    ANALYSIS_DOCUMENT_EXTRACT_ROUTE, WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
    WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER, WENDAO_DOCUMENT_EXTRACT_MODE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER, WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_DOCUMENT_EXTRACT_WAIT_MS_HEADER, WENDAO_SCHEMA_VERSION_HEADER,
};

const DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug)]
struct PerfConfig {
    endpoint: String,
    source: String,
    output_dir: String,
    iterations: usize,
    concurrency: usize,
    force_first: bool,
    mode: String,
    wait_ms: u64,
    report_path: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PerfReport {
    schema: &'static str,
    endpoint: String,
    source: String,
    output_dir: String,
    iterations: usize,
    concurrency: usize,
    request_count: usize,
    force_first: bool,
    mode: String,
    wait_ms: u64,
    row_count: usize,
    batch_count: usize,
    arrow_ipc_bytes: usize,
    error_row_count: usize,
    status_counts: BTreeMap<String, usize>,
    wall_time_ms: f64,
    latencies_ms: Vec<f64>,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires an external Python document extraction Flight service"]
async fn document_extract_python_flight_perf_smoke() -> Result<(), String> {
    let config = perf_config_from_env()?;
    if config.force_first && config.concurrency > 1 {
        return Err(
            "WENDAO_DOCUMENT_EXTRACT_PERF_FORCE_FIRST cannot be combined with concurrency > 1"
                .to_string(),
        );
    }

    let mut latencies_ms = Vec::with_capacity(config.iterations * config.concurrency);
    let mut last_batches = Vec::new();
    let channel = connect_document_extract(&config.endpoint).await?;
    let overall_started = Instant::now();

    for iteration in 0..config.iterations {
        let requests = (0..config.concurrency).map(|_| async {
            let started = Instant::now();
            let batches = request_document_extract(&config, iteration, channel.clone()).await?;
            Ok::<_, String>((started.elapsed().as_secs_f64() * 1000.0, batches))
        });
        for (latency_ms, batches) in try_join_all(requests).await? {
            latencies_ms.push(latency_ms);
            last_batches = batches;
        }
    }
    let wall_time_ms = overall_started.elapsed().as_secs_f64() * 1000.0;

    let report = PerfReport {
        schema: "xiuxian_wendao.document_extract_perf_probe.v1",
        endpoint: config.endpoint,
        source: config.source,
        output_dir: config.output_dir,
        iterations: config.iterations,
        concurrency: config.concurrency,
        request_count: config.iterations * config.concurrency,
        force_first: config.force_first,
        mode: config.mode,
        wait_ms: config.wait_ms,
        row_count: last_batches.iter().map(RecordBatch::num_rows).sum(),
        batch_count: last_batches.len(),
        arrow_ipc_bytes: encode_batches(&last_batches)?.len(),
        error_row_count: error_row_count(&last_batches)?,
        status_counts: status_counts(&last_batches)?,
        wall_time_ms,
        latencies_ms,
    };
    let report_json = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    if let Some(report_path) = config.report_path {
        if let Some(parent) = report_path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::write(&report_path, report_json.as_bytes()).map_err(|error| error.to_string())?;
    }
    println!("{report_json}");
    Ok(())
}

fn perf_config_from_env() -> Result<PerfConfig, String> {
    let source = std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_SOURCE")
        .map_err(|_| "WENDAO_DOCUMENT_EXTRACT_PERF_SOURCE is required".to_string())?;
    let output_dir = std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_OUTPUT_DIR")
        .map_err(|_| "WENDAO_DOCUMENT_EXTRACT_PERF_OUTPUT_DIR is required".to_string())?;
    let iterations = std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_ITERATIONS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(3)
        .max(1);
    let concurrency = std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_CONCURRENCY")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1)
        .max(1);
    Ok(PerfConfig {
        endpoint: std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_ENDPOINT")
            .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string()),
        source,
        output_dir,
        iterations,
        concurrency,
        force_first: std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_FORCE_FIRST")
            .is_ok_and(|value| matches!(value.as_str(), "1" | "true" | "yes")),
        mode: std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_MODE")
            .unwrap_or_else(|_| "sync".to_string()),
        wait_ms: std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_WAIT_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or_default(),
        report_path: std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_REPORT")
            .ok()
            .map(PathBuf::from),
    })
}

async fn request_document_extract(
    config: &PerfConfig,
    iteration: usize,
    channel: Channel,
) -> Result<Vec<RecordBatch>, String> {
    let inner_client = TonicFlightServiceClient::new(channel)
        .max_encoding_message_size(DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES)
        .max_decoding_message_size(DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES);
    let mut client = FlightClient::new_from_inner(inner_client);
    client
        .add_header(WENDAO_SCHEMA_VERSION_HEADER, "v2")
        .map_err(|error| error.to_string())?;
    client
        .add_header(
            WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
            config.source.as_str(),
        )
        .map_err(|error| error.to_string())?;
    client
        .add_header(
            WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
            config.output_dir.as_str(),
        )
        .map_err(|error| error.to_string())?;
    client
        .add_header(
            WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
            if config.force_first && iteration == 0 {
                "true"
            } else {
                "false"
            },
        )
        .map_err(|error| error.to_string())?;
    client
        .add_header(WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER, "true")
        .map_err(|error| error.to_string())?;
    client
        .add_header(WENDAO_DOCUMENT_EXTRACT_MODE_HEADER, config.mode.as_str())
        .map_err(|error| error.to_string())?;
    let wait_ms = config.wait_ms.to_string();
    client
        .add_header(WENDAO_DOCUMENT_EXTRACT_WAIT_MS_HEADER, wait_ms.as_str())
        .map_err(|error| error.to_string())?;

    let descriptor = FlightDescriptor::new_path(
        ANALYSIS_DOCUMENT_EXTRACT_ROUTE
            .trim_start_matches('/')
            .split('/')
            .map(ToString::to_string)
            .collect(),
    );
    let flight_info = client
        .get_flight_info(descriptor)
        .await
        .map_err(|error| error.to_string())?;
    let ticket = flight_info
        .endpoint
        .first()
        .and_then(|endpoint| endpoint.ticket.clone())
        .ok_or_else(|| "document extract flight info missing ticket".to_string())?;
    client
        .do_get(ticket)
        .await
        .map_err(|error| error.to_string())?
        .try_collect()
        .await
        .map_err(|error| error.to_string())
}

async fn connect_document_extract(endpoint_url: &str) -> Result<Channel, String> {
    let endpoint = Endpoint::from_shared(endpoint_url.to_string())
        .map_err(|error| format!("invalid document extract endpoint `{endpoint_url}`: {error}"))?;
    endpoint.connect().await.map_err(|error| {
        format!("failed to connect to document extract endpoint `{endpoint_url}`: {error}")
    })
}

fn encode_batches(batches: &[RecordBatch]) -> Result<Vec<u8>, String> {
    let Some(first_batch) = batches.first() else {
        return Ok(Vec::new());
    };
    let mut encoded = Vec::new();
    {
        let mut writer = StreamWriter::try_new(&mut encoded, &first_batch.schema())
            .map_err(|error| error.to_string())?;
        for batch in batches {
            writer.write(batch).map_err(|error| error.to_string())?;
        }
        writer.finish().map_err(|error| error.to_string())?;
    }
    Ok(encoded)
}

fn error_row_count(batches: &[RecordBatch]) -> Result<usize, String> {
    Ok(status_counts(batches)?
        .get("error")
        .copied()
        .unwrap_or_default())
}

fn status_counts(batches: &[RecordBatch]) -> Result<BTreeMap<String, usize>, String> {
    let mut counts = BTreeMap::new();
    for batch in batches {
        let Some(status_column) = batch.column_by_name("status") else {
            continue;
        };
        let Some(status_array) = status_column.as_any().downcast_ref::<StringArray>() else {
            return Err("document extract status column is not a string array".to_string());
        };
        for row in 0..status_array.len() {
            let status = if status_array.is_null(row) {
                ""
            } else {
                status_array.value(row)
            };
            *counts.entry(status.to_string()).or_insert(0) += 1;
        }
    }
    Ok(counts)
}
