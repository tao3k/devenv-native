use std::sync::Arc;
use std::time::Duration;

use arrow_flight::FlightDescriptor;
use arrow_flight::client::FlightClient;
use arrow_flight::encode::FlightDataEncoderBuilder;
use arrow_flight::flight_service_client::FlightServiceClient as TonicFlightServiceClient;
use arrow_schema::DataType;
use futures::{TryStreamExt, stream};
use tokio::sync::{Mutex, Semaphore};
use tonic::transport::Endpoint;
use xiuxian_vector_store::EngineRecordBatch;

use super::query_contract::{
    RERANK_REQUEST_EMBEDDING_COLUMN, RERANK_ROUTE, WENDAO_RERANK_DIMENSION_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER, flight_descriptor_path, normalize_flight_route,
};

/// Lazy Arrow Flight client aligned to the workspace Arrow Flight transport
/// line.
///
/// This matches the `WendaoSearch` parser-summary server default so dense
/// parser-summary responses do not silently truncate below the server ceiling.
pub(crate) const DEFAULT_FLIGHT_MESSAGE_SIZE_BYTES: usize = 256 * 1024 * 1024;

#[derive(Clone)]
pub(crate) struct ArrowFlightTransportClient {
    base_url: String,
    route: String,
    schema_version: String,
    #[cfg(test)]
    timeout: Duration,
    endpoint: Endpoint,
    client: Arc<Mutex<Option<FlightClient>>>,
    in_flight_gate: Arc<Semaphore>,
}

impl ArrowFlightTransportClient {
    /// Create one lazy Arrow Flight client.
    ///
    /// # Errors
    ///
    /// Returns an error when the base URL, route, schema version, or timeout
    /// cannot be represented as a valid Flight transport configuration.
    pub(crate) fn new(
        base_url: impl Into<String>,
        route: impl Into<String>,
        schema_version: impl Into<String>,
        timeout: Duration,
        max_in_flight_requests: usize,
    ) -> Result<Self, String> {
        if timeout.is_zero() {
            return Err("Arrow Flight timeout must be greater than zero".to_string());
        }
        if max_in_flight_requests == 0 {
            return Err(
                "Arrow Flight max_in_flight_requests must be greater than zero".to_string(),
            );
        }

        let base_url = base_url.into();
        let route = normalize_flight_route(route.into())?;
        let schema_version = schema_version.into();
        if schema_version.trim().is_empty() {
            return Err("Arrow Flight schema version must not be blank".to_string());
        }

        let endpoint = Endpoint::from_shared(base_url.clone())
            .map_err(|error| format!("invalid Arrow Flight base URL `{base_url}`: {error}"))?
            .connect_timeout(timeout)
            .timeout(timeout);

        Ok(Self {
            base_url,
            route,
            schema_version,
            #[cfg(test)]
            timeout,
            endpoint,
            client: Arc::new(Mutex::new(None)),
            in_flight_gate: Arc::new(Semaphore::new(max_in_flight_requests)),
        })
    }

    /// Return the configured Flight endpoint base URL.
    #[must_use]
    pub(crate) fn base_url(&self) -> &str {
        self.base_url.as_str()
    }

    /// Return the configured Flight descriptor route.
    #[must_use]
    pub(crate) fn route(&self) -> &str {
        self.route.as_str()
    }

    /// Return the configured schema version metadata value.
    #[must_use]
    #[cfg(test)]
    pub(crate) fn schema_version(&self) -> &str {
        self.schema_version.as_str()
    }

    /// Return the configured request timeout.
    #[must_use]
    #[cfg(test)]
    pub(crate) fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Return the configured in-flight request budget.
    #[must_use]
    #[cfg(test)]
    pub(crate) fn max_in_flight_requests(&self) -> usize {
        self.in_flight_gate.available_permits()
    }

    /// Return the shared in-flight request gate.
    #[must_use]
    #[cfg(test)]
    pub(crate) fn request_gate(&self) -> Arc<Semaphore> {
        Arc::clone(&self.in_flight_gate)
    }

    /// Send one Arrow engine batch through the Flight transport.
    ///
    /// # Errors
    ///
    /// Returns an error when the request cannot be converted onto the
    /// workspace Arrow Flight transport line or when the Flight request fails.
    pub(crate) async fn process_batch(
        &self,
        batch: &EngineRecordBatch,
    ) -> Result<Vec<EngineRecordBatch>, String> {
        self.process_batches(std::slice::from_ref(batch)).await
    }

    /// Send multiple Arrow engine batches through the Flight transport.
    ///
    /// # Errors
    ///
    /// Returns an error when the request cannot be converted onto the
    /// workspace Arrow Flight transport line or when the Flight request fails.
    pub(crate) async fn process_batches(
        &self,
        batches: &[EngineRecordBatch],
    ) -> Result<Vec<EngineRecordBatch>, String> {
        if batches.is_empty() {
            return Err("Arrow Flight request batches cannot be empty".to_string());
        }
        let _in_flight_request_permit = Arc::clone(&self.in_flight_gate)
            .acquire_owned()
            .await
            .map_err(|_| "Arrow Flight in-flight request gate unexpectedly closed".to_string())?;

        let first_batch = batches
            .first()
            .ok_or_else(|| "Arrow Flight request batches cannot be empty".to_string())?;
        let rerank_dimension_header = rerank_dimension_header(self.route.as_str(), batches)?;
        let request_batches = batches.to_vec();
        let request_stream = FlightDataEncoderBuilder::new()
            .with_schema(first_batch.schema())
            .with_flight_descriptor(Some(flight_descriptor(self.route.as_str())))
            .with_max_flight_data_size(DEFAULT_FLIGHT_MESSAGE_SIZE_BYTES)
            .build(stream::iter(request_batches.into_iter().map(
                Ok::<EngineRecordBatch, arrow_flight::error::FlightError>,
            )));

        let response = {
            let mut client = self.client.lock().await;
            if client.is_none() {
                let channel =
                    self.endpoint.clone().connect().await.map_err(|error| {
                        format!("failed to connect Arrow Flight endpoint: {error}")
                    })?;
                let inner_client = TonicFlightServiceClient::new(channel)
                    .max_encoding_message_size(DEFAULT_FLIGHT_MESSAGE_SIZE_BYTES)
                    .max_decoding_message_size(DEFAULT_FLIGHT_MESSAGE_SIZE_BYTES);
                let mut flight_client = FlightClient::new_from_inner(inner_client);
                flight_client
                    .add_header(WENDAO_SCHEMA_VERSION_HEADER, self.schema_version.as_str())
                    .map_err(|error| {
                        format!("invalid Arrow Flight schema-version metadata: {error}")
                    })?;
                if let Some(rerank_dimension_header) = rerank_dimension_header.as_deref() {
                    flight_client
                        .add_header(WENDAO_RERANK_DIMENSION_HEADER, rerank_dimension_header)
                        .map_err(|error| {
                            format!("invalid Arrow Flight rerank-dimension metadata: {error}")
                        })?;
                }
                *client = Some(flight_client);
            }

            let Some(flight_client) = client.as_mut() else {
                return Err(
                    "Arrow Flight client initialization unexpectedly returned no client"
                        .to_string(),
                );
            };
            flight_client
                .do_exchange(request_stream)
                .await
                .map_err(|error| format!("Arrow Flight request failed: {error}"))?
        };

        let response_batches = response
            .try_collect::<Vec<EngineRecordBatch>>()
            .await
            .map_err(|error| format!("failed to decode Arrow Flight response: {error}"))?;
        Ok(response_batches)
    }
}

fn flight_descriptor(route: &str) -> FlightDescriptor {
    let path = flight_descriptor_path(route).unwrap_or_else(|error| {
        panic!("flight descriptor route should already be normalized: {error}")
    });
    FlightDescriptor::new_path(path)
}

fn rerank_dimension_header(
    route: &str,
    request_batches: &[EngineRecordBatch],
) -> Result<Option<String>, String> {
    if route != RERANK_ROUTE {
        return Ok(None);
    }

    let first_batch = request_batches
        .first()
        .ok_or_else(|| "Arrow Flight request batches cannot be empty".to_string())?;
    let embedding_column = first_batch
        .column_by_name(RERANK_REQUEST_EMBEDDING_COLUMN)
        .ok_or_else(|| {
            format!("rerank Flight request missing `{RERANK_REQUEST_EMBEDDING_COLUMN}` column")
        })?;
    match embedding_column.data_type() {
        DataType::FixedSizeList(_, dimension) if *dimension > 0 => Ok(Some(dimension.to_string())),
        other => Err(format!(
            "rerank Flight request column `{RERANK_REQUEST_EMBEDDING_COLUMN}` must be FixedSizeList, found {other:?}"
        )),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/transport/flight.rs"]
mod tests;
