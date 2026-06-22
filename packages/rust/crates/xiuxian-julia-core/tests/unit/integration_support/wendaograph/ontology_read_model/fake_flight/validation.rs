use arrow::array::{Array, BinaryArray};
use arrow::record_batch::RecordBatch;
use arrow_flight::FlightData;
use arrow_flight::decode::FlightRecordBatchStream;
use futures::{StreamExt, TryStreamExt};
use tonic::{Request, Status};
use xiuxian_wendao_runtime::transport::WENDAO_SCHEMA_VERSION_HEADER;

use crate::integration_support::wendaograph::{
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION,
    WENDAO_GRAPH_ONTOLOGY_SEMANTIC_OBJECTS_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_SEMANTIC_PROJECTION_STATE_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_SEMANTIC_RELATIONS_PAYLOAD_COLUMN,
};

pub(super) fn validate_schema_version(
    request: &Request<tonic::Streaming<FlightData>>,
) -> Result<(), Status> {
    let schema_version = request
        .metadata()
        .get(WENDAO_SCHEMA_VERSION_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if schema_version == WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(Status::invalid_argument(format!(
            "unexpected schema version header: {schema_version}"
        )))
    }
}

pub(super) async fn decode_and_validate_request_bundle(
    request: Request<tonic::Streaming<FlightData>>,
) -> Result<(), Status> {
    let stream = request
        .into_inner()
        .map(|frame| frame.map_err(arrow_flight::error::FlightError::from))
        .try_filter(|frame| futures::future::ready(!frame.data_header.is_empty()));
    let mut batch_stream = FlightRecordBatchStream::new_from_flight_data(stream);
    let mut decoded_batches = Vec::new();
    while let Some(batch) = batch_stream
        .try_next()
        .await
        .map_err(|error| Status::invalid_argument(error.to_string()))?
    {
        decoded_batches.push(batch);
    }
    let [batch] = decoded_batches.as_slice() else {
        return Err(Status::invalid_argument(format!(
            "expected one request bundle batch, got {}",
            decoded_batches.len()
        )));
    };

    validate_payload_column(batch, WENDAO_GRAPH_ONTOLOGY_SEMANTIC_OBJECTS_PAYLOAD_COLUMN)?;
    validate_payload_column(
        batch,
        WENDAO_GRAPH_ONTOLOGY_SEMANTIC_RELATIONS_PAYLOAD_COLUMN,
    )?;
    validate_payload_column(
        batch,
        WENDAO_GRAPH_ONTOLOGY_SEMANTIC_PROJECTION_STATE_PAYLOAD_COLUMN,
    )
}

fn validate_payload_column(batch: &RecordBatch, column_name: &str) -> Result<(), Status> {
    let column = batch
        .column_by_name(column_name)
        .and_then(|column| column.as_any().downcast_ref::<BinaryArray>())
        .ok_or_else(|| {
            Status::invalid_argument(format!("missing binary column `{column_name}`"))
        })?;
    if column.is_empty() || column.value(0).is_empty() {
        return Err(Status::invalid_argument(format!(
            "empty binary column `{column_name}`"
        )));
    }
    Ok(())
}
