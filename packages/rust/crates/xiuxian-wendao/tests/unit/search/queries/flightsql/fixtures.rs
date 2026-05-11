use arrow::array::Array;
use arrow::record_batch::RecordBatch;
use arrow::util::display::array_value_to_string;
use arrow_flight::decode::FlightRecordBatchStream;
use arrow_flight::flight_service_server::FlightService;
use arrow_flight::sql::ProstMessageExt;
use arrow_flight::{FlightData, FlightDescriptor};
use prost::Message;
use serde_json::{Map, Value, json};
use tempfile::TempDir;
use tokio_stream::StreamExt;
use tonic::{Request, Status};

use crate::repo_index::RepoCodeDocument;
use crate::search::SearchPlaneService;
use crate::search::contracts::{AstSearchHit, StudioNavigationTarget};
use crate::search::queries::flightsql::StudioFlightSqlService;
use crate::search::queries::tests::fixtures as shared_fixtures;
#[cfg(feature = "duckdb")]
use crate::set_link_graph_wendao_config_override;
#[cfg(feature = "duckdb")]
use std::fs;

pub(super) fn fixture_service(temp_dir: &TempDir) -> SearchPlaneService {
    shared_fixtures::fixture_service(temp_dir, "xiuxian:test:studio_flightsql")
}

#[cfg(feature = "duckdb")]
pub(super) fn write_search_duckdb_runtime_override(
    body: &str,
) -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(&config_path, body)?;
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());
    Ok(temp)
}

pub(super) fn repo_document(
    path: &str,
    contents: &str,
    language: &str,
    modified_unix_ms: u64,
) -> RepoCodeDocument {
    RepoCodeDocument {
        path: path.to_string().into(),
        language: Some(language.to_string()),
        contents: contents.into(),
        size_bytes: u64::try_from(contents.len()).unwrap_or(u64::MAX),
        modified_unix_ms,
    }
}

pub(super) async fn publish_repo_content_chunks(
    service: &SearchPlaneService,
    repo_id: &str,
    documents: &[RepoCodeDocument],
    source_revision: &str,
) {
    service
        .publish_repo_content_chunks_with_revision(repo_id, documents, Some(source_revision))
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks: {error}"));
}

pub(super) fn sample_local_symbol_hit(name: &str, path: &str, line_start: usize) -> AstSearchHit {
    AstSearchHit {
        name: name.to_string(),
        signature: format!("fn {name}()"),
        path: path.to_string().into(),
        language: "rust".to_string(),
        crate_name: "kernel".to_string(),
        project_name: None,
        root_label: None,
        node_kind: None,
        owner_title: None,
        navigation_target: StudioNavigationTarget {
            path: path.to_string().into(),
            category: "symbol".to_string(),
            project_name: None,
            root_label: None,
            line: Some(line_start),
            line_end: Some(line_start),
            column: Some(1),
        },
        line_start,
        line_end: line_start,
        score: 0.0,
    }
}

pub(super) async fn publish_local_symbol_hits(
    service: &SearchPlaneService,
    build_id: &str,
    hits: &[AstSearchHit],
) {
    service
        .publish_local_symbol_hits(build_id, hits)
        .await
        .unwrap_or_else(|error| panic!("publish local symbol hits: {error}"));
}

pub(super) async fn collect_flight_frames<S>(stream: S) -> Vec<Result<FlightData, Status>>
where
    S: tokio_stream::Stream<Item = Result<FlightData, Status>>,
{
    stream.collect::<Vec<_>>().await
}

pub(super) async fn decode_flight_batches(
    frames: Vec<Result<FlightData, Status>>,
) -> Vec<RecordBatch> {
    let batch_stream = FlightRecordBatchStream::new_from_flight_data(tokio_stream::iter(
        frames
            .into_iter()
            .map(|frame| frame.map_err(arrow_flight::error::FlightError::from)),
    ));
    batch_stream
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|batch| batch.unwrap_or_else(|error| panic!("decode Flight batches: {error}")))
        .collect()
}

pub(super) async fn fetch_command_batches<T>(
    service: &StudioFlightSqlService,
    command: T,
) -> Vec<RecordBatch>
where
    T: Message + ProstMessageExt,
{
    let descriptor = FlightDescriptor::new_cmd(command.as_any().encode_to_vec());
    let flight_info = FlightService::get_flight_info(service, Request::new(descriptor))
        .await
        .unwrap_or_else(|error| panic!("get discovery flight info: {error}"))
        .into_inner();
    let ticket = flight_info
        .endpoint
        .first()
        .and_then(|endpoint| endpoint.ticket.clone())
        .unwrap_or_else(|| panic!("discovery flight info should expose a ticket"));
    let frames = collect_flight_frames(
        FlightService::do_get(service, Request::new(ticket))
            .await
            .unwrap_or_else(|error| panic!("do_get discovery: {error}"))
            .into_inner(),
    )
    .await;
    decode_flight_batches(frames).await
}

pub(super) fn string_value(batch: &RecordBatch, column_name: &str, row_index: usize) -> String {
    let column = batch
        .column_by_name(column_name)
        .unwrap_or_else(|| panic!("missing column `{column_name}`"));
    array_value_to_string(column.as_ref(), row_index)
        .unwrap_or_else(|error| panic!("decode column `{column_name}` at row {row_index}: {error}"))
}

pub(super) fn string_column_values(batch: &RecordBatch, column_name: &str) -> Vec<String> {
    (0..batch.num_rows())
        .map(|row_index| string_value(batch, column_name, row_index))
        .collect()
}

pub(super) fn flight_batches_snapshot(batches: &[RecordBatch]) -> Value {
    Value::Array(batches.iter().map(batch_snapshot).collect())
}

pub(super) use shared_fixtures::{publish_reference_hits, sample_hit};

fn batch_snapshot(batch: &RecordBatch) -> Value {
    let schema = batch
        .schema()
        .fields()
        .iter()
        .map(|field| {
            json!({
                "name": field.name(),
                "data_type": field.data_type().to_string(),
            })
        })
        .collect::<Vec<_>>();
    let rows = (0..batch.num_rows())
        .map(|row_index| {
            let mut row = Map::new();
            for (column_index, field) in batch.schema().fields().iter().enumerate() {
                let column = batch.column(column_index);
                let value = if column.is_null(row_index) {
                    Value::Null
                } else {
                    Value::String(
                        array_value_to_string(column.as_ref(), row_index).unwrap_or_else(|error| {
                            panic!(
                                "snapshot value for `{}` at row {}: {error}",
                                field.name(),
                                row_index
                            )
                        }),
                    )
                };
                row.insert(field.name().clone(), value);
            }
            Value::Object(row)
        })
        .collect::<Vec<_>>();

    json!({
        "schema": schema,
        "rows": rows,
    })
}
