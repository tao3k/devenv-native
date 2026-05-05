use std::collections::HashSet;
use std::sync::atomic::Ordering;

use arrow::record_batch::RecordBatch as EngineRecordBatch;
use arrow_flight::FlightDescriptor;
use arrow_flight::client::FlightClient;
use arrow_flight::flight_service_client::FlightServiceClient as TonicFlightServiceClient;
use futures::TryStreamExt;
use tonic::transport::{Channel, Endpoint};
use xiuxian_wendao_server::transport::{
    ANALYSIS_DOCUMENT_EXTRACT_ROUTE, WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
    WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER, WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
    WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER, WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER,
};

use super::{
    DEFAULT_DOCUMENT_EXTRACT_ENDPOINT, DOCUMENT_EXTRACT_ENDPOINT_ENV,
    DOCUMENT_EXTRACT_ENDPOINTS_ENV, DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES,
    StudioDocumentExtractFlightRouteProvider,
};

impl StudioDocumentExtractFlightRouteProvider {
    pub(super) async fn channel_for_endpoint(&self, endpoint_url: &str) -> Result<Channel, String> {
        {
            let channels = self.runtime.channels.lock().await;
            if let Some(channel) = channels.get(endpoint_url) {
                return Ok(channel.clone());
            }
        }

        let endpoint = Endpoint::from_shared(endpoint_url.to_string()).map_err(|error| {
            format!("invalid document extract endpoint `{endpoint_url}`: {error}")
        })?;

        let channel = endpoint.connect().await.map_err(|error| {
            format!("failed to connect to document extract endpoint `{endpoint_url}`: {error}")
        })?;

        let mut channels = self.runtime.channels.lock().await;
        Ok(channels
            .entry(endpoint_url.to_string())
            .or_insert_with(|| channel.clone())
            .clone())
    }

    pub(super) async fn request_python_document_extract(
        &self,
        source_path: &str,
        output_dir: &str,
        force: bool,
        error_row: bool,
        profile: &str,
    ) -> Result<Vec<EngineRecordBatch>, String> {
        let endpoint_url = self.document_extract_endpoint_url()?;

        let channel = self.channel_for_endpoint(endpoint_url.as_str()).await?;

        let inner_client = TonicFlightServiceClient::new(channel)
            .max_encoding_message_size(DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES)
            .max_decoding_message_size(DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES);
        let mut client = FlightClient::new_from_inner(inner_client);
        client
            .add_header(WENDAO_SCHEMA_VERSION_HEADER, "v2")
            .map_err(|error| format!("invalid schema version header: {error}"))?;
        client
            .add_header(WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER, source_path)
            .map_err(|error| format!("invalid source path header: {error}"))?;
        client
            .add_header(WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER, output_dir)
            .map_err(|error| format!("invalid output dir header: {error}"))?;
        client
            .add_header(
                WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
                if force { "true" } else { "false" },
            )
            .map_err(|error| format!("invalid force header: {error}"))?;
        client
            .add_header(
                WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
                if error_row { "true" } else { "false" },
            )
            .map_err(|error| format!("invalid error-row header: {error}"))?;
        client
            .add_header(WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER, profile)
            .map_err(|error| format!("invalid profile header: {error}"))?;

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

        let stream = client
            .do_get(ticket)
            .await
            .map_err(|error| format!("document extract do_get failed: {error}"))?;

        let engine_batches: Vec<EngineRecordBatch> = stream
            .try_collect()
            .await
            .map_err(|error| format!("document extract stream decode failed: {error}"))?;

        if engine_batches.is_empty() {
            return Err("document extract returned no record batches".to_string());
        }
        Ok(engine_batches)
    }

    fn document_extract_endpoint_url(&self) -> Result<String, String> {
        let default_endpoint = document_extract_default_endpoint_with_lookup(
            self.configured_default_endpoint.as_deref(),
            &|key| std::env::var(key).ok(),
        );
        let endpoint_urls = document_extract_endpoint_urls(default_endpoint.as_str());
        let request_index = self
            .runtime
            .endpoint_round_robin
            .fetch_add(1, Ordering::Relaxed);
        let endpoint_index = endpoint_index_for_request(request_index, endpoint_urls.len())?;
        Ok(endpoint_urls[endpoint_index].clone())
    }
}

pub(super) fn document_extract_default_endpoint_with_lookup(
    configured_default_endpoint: Option<&str>,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> String {
    configured_default_endpoint
        .and_then(normalize_endpoint)
        .or_else(|| {
            lookup(DOCUMENT_EXTRACT_ENDPOINT_ENV).and_then(|value| normalize_endpoint(&value))
        })
        .unwrap_or_else(|| DEFAULT_DOCUMENT_EXTRACT_ENDPOINT.to_string())
}

pub(super) fn document_extract_endpoint_urls(default_endpoint: &str) -> Vec<String> {
    document_extract_endpoint_urls_with_lookup(default_endpoint, &|key| std::env::var(key).ok())
}

pub(super) fn document_extract_endpoint_urls_with_lookup(
    default_endpoint: &str,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Vec<String> {
    let configured =
        lookup(DOCUMENT_EXTRACT_ENDPOINTS_ENV).unwrap_or_else(|| default_endpoint.to_string());
    let mut seen = HashSet::new();
    let endpoints = configured
        .split(|character: char| character == ',' || character == ';' || character.is_whitespace())
        .filter_map(normalize_endpoint)
        .filter(|endpoint| seen.insert(endpoint.clone()))
        .collect::<Vec<_>>();
    if endpoints.is_empty() {
        normalize_endpoint(default_endpoint)
            .into_iter()
            .collect::<Vec<_>>()
    } else {
        endpoints
    }
}

pub(super) fn endpoint_index_for_request(
    request_index: usize,
    endpoint_count: usize,
) -> Result<usize, String> {
    if endpoint_count == 0 {
        return Err("document extract endpoint pool cannot be empty".to_string());
    }
    Ok(request_index % endpoint_count)
}

fn normalize_endpoint(endpoint: &str) -> Option<String> {
    let endpoint = endpoint.trim().trim_end_matches('/');
    (!endpoint.is_empty()).then(|| endpoint.to_string())
}
