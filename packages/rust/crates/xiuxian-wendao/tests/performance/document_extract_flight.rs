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
use serde::{Deserialize, Serialize};
use tonic::transport::{Channel, Endpoint};
use xiuxian_wendao_runtime::transport::{
    ANALYSIS_DOCUMENT_EXTRACT_ROUTE, WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
    WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER, WENDAO_DOCUMENT_EXTRACT_MODE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER, WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_UTF8_HEX_HEADER, WENDAO_DOCUMENT_EXTRACT_WAIT_MS_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER, encode_document_extract_source_path_utf8_hex,
};

use super::support::document_extract_artifacts::{ArtifactReport, inspect_artifacts};

const DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug)]
struct PerfConfig {
    endpoint: String,
    source: String,
    output_dir: String,
    inputs: Vec<PerfInput>,
    iterations: usize,
    force_first: bool,
    mode: String,
    wait_ms: u64,
    structure_baseline_root: Option<PathBuf>,
    report_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PerfInput {
    source: String,
    output_dir: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PerfReport {
    schema: &'static str,
    endpoint: String,
    source: String,
    output_dir: String,
    sources: Vec<String>,
    output_dirs: Vec<String>,
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
    artifact_reports: Vec<ArtifactReport>,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires an external Python document extraction Flight service"]
async fn document_extract_python_flight_perf_smoke() -> Result<(), String> {
    let config = perf_config_from_env()?;
    let request_concurrency = config.inputs.len();
    if config.force_first && request_concurrency > 1 {
        return Err(
            "WENDAO_DOCUMENT_EXTRACT_PERF_FORCE_FIRST cannot be combined with concurrency > 1"
                .to_string(),
        );
    }

    let mut latencies_ms = Vec::with_capacity(config.iterations * request_concurrency);
    let mut last_batches = Vec::new();
    let channel = connect_document_extract(&config.endpoint).await?;
    let overall_started = Instant::now();

    for iteration in 0..config.iterations {
        let requests = config.inputs.iter().map(|input| async {
            let started = Instant::now();
            let batches =
                request_document_extract(&config, input, iteration, channel.clone()).await?;
            Ok::<_, String>((started.elapsed().as_secs_f64() * 1000.0, batches))
        });
        let mut iteration_batches = Vec::new();
        for (latency_ms, batches) in try_join_all(requests).await? {
            latencies_ms.push(latency_ms);
            iteration_batches.extend(batches);
        }
        last_batches = iteration_batches;
    }
    let wall_time_ms = overall_started.elapsed().as_secs_f64() * 1000.0;
    let artifact_reports = inspect_artifacts(
        config
            .inputs
            .iter()
            .map(|input| (input.source.as_str(), input.output_dir.as_str())),
        config.structure_baseline_root.as_deref(),
    );

    let report = PerfReport {
        schema: "xiuxian_wendao.document_extract_perf_probe.v1",
        endpoint: config.endpoint,
        source: config.source,
        output_dir: config.output_dir,
        sources: config
            .inputs
            .iter()
            .map(|input| input.source.clone())
            .collect(),
        output_dirs: config
            .inputs
            .iter()
            .map(|input| input.output_dir.clone())
            .collect(),
        iterations: config.iterations,
        concurrency: request_concurrency,
        request_count: config.iterations * request_concurrency,
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
        artifact_reports,
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
    let source = std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_SOURCE").ok();
    let output_dir = std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_OUTPUT_DIR").ok();
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
    let inputs = perf_inputs_from_env(source.as_deref(), output_dir.as_deref(), concurrency)?;
    let report_source = if inputs.len() == 1 {
        inputs[0].source.clone()
    } else {
        "<multiple>".to_string()
    };
    let report_output_dir = if inputs.len() == 1 {
        inputs[0].output_dir.clone()
    } else {
        "<multiple>".to_string()
    };
    Ok(PerfConfig {
        endpoint: std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_ENDPOINT")
            .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string()),
        source: report_source,
        output_dir: report_output_dir,
        inputs,
        iterations,
        force_first: std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_FORCE_FIRST")
            .is_ok_and(|value| matches!(value.as_str(), "1" | "true" | "yes")),
        mode: std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_MODE")
            .unwrap_or_else(|_| "sync".to_string()),
        wait_ms: std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_WAIT_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or_default(),
        structure_baseline_root: std::env::var(
            "WENDAO_DOCUMENT_EXTRACT_PERF_STRUCTURE_BASELINE_ROOT",
        )
        .ok()
        .map(PathBuf::from),
        report_path: std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_REPORT")
            .ok()
            .map(PathBuf::from),
    })
}

fn perf_inputs_from_env(
    source: Option<&str>,
    output_dir: Option<&str>,
    concurrency: usize,
) -> Result<Vec<PerfInput>, String> {
    if let Ok(inputs_json) = std::env::var("WENDAO_DOCUMENT_EXTRACT_PERF_INPUTS_JSON") {
        let inputs: Vec<PerfInput> =
            serde_json::from_str(&inputs_json).map_err(|error| error.to_string())?;
        if inputs.is_empty() {
            return Err("WENDAO_DOCUMENT_EXTRACT_PERF_INPUTS_JSON must not be empty".to_string());
        }
        return Ok(inputs);
    }
    let source =
        source.ok_or_else(|| "WENDAO_DOCUMENT_EXTRACT_PERF_SOURCE is required".to_string())?;
    let output_dir = output_dir
        .ok_or_else(|| "WENDAO_DOCUMENT_EXTRACT_PERF_OUTPUT_DIR is required".to_string())?;
    Ok((0..concurrency)
        .map(|_| PerfInput {
            source: source.to_string(),
            output_dir: output_dir.to_string(),
        })
        .collect())
}

async fn request_document_extract(
    config: &PerfConfig,
    input: &PerfInput,
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
    add_source_path_headers(&mut client, input.source.as_str())?;
    client
        .add_header(
            WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
            input.output_dir.as_str(),
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
        .map_err(|error| format!("document extract get_flight_info failed: {error}"))?;
    let ticket = flight_info
        .endpoint
        .first()
        .and_then(|endpoint| endpoint.ticket.clone())
        .ok_or_else(|| "document extract flight info missing ticket".to_string())?;
    client
        .do_get(ticket)
        .await
        .map_err(|error| format!("document extract do_get failed: {error}"))?
        .try_collect()
        .await
        .map_err(|error| format!("document extract stream collect failed: {error}"))
}

fn add_source_path_headers(client: &mut FlightClient, source_path: &str) -> Result<(), String> {
    let encoded = encode_document_extract_source_path_utf8_hex(source_path);
    client
        .add_header(
            WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_UTF8_HEX_HEADER,
            encoded.as_str(),
        )
        .map_err(|error| error.to_string())?;
    if source_path.is_ascii() {
        client
            .add_header(WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER, source_path)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
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
    string_counts(batches, "status")
}

fn string_counts(
    batches: &[RecordBatch],
    column_name: &str,
) -> Result<BTreeMap<String, usize>, String> {
    let mut counts = BTreeMap::new();
    for batch in batches {
        let Some(column) = batch.column_by_name(column_name) else {
            continue;
        };
        let Some(array) = column.as_any().downcast_ref::<StringArray>() else {
            return Err(format!(
                "document extract `{column_name}` column is not a string array"
            ));
        };
        for row in 0..array.len() {
            let value = if array.is_null(row) {
                ""
            } else {
                array.value(row)
            };
            *counts.entry(value.to_string()).or_insert(0) += 1;
        }
    }
    Ok(counts)
}
