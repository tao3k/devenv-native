use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use arrow::record_batch::RecordBatch as EngineRecordBatch;
use arrow_flight::FlightDescriptor;
use arrow_flight::client::FlightClient;
use arrow_flight::flight_service_client::FlightServiceClient as TonicFlightServiceClient;
use futures::TryStreamExt;
use tokio::sync::{OwnedSemaphorePermit, TryAcquireError};
use tonic::transport::{Channel, Endpoint};
use xiuxian_llm::model_routing::wendao_model_route_metadata;
use xiuxian_polyglot_orchestrator::{
    DocumentExtractPressureEvidenceInput, document_extract_pressure_evidence,
    document_extract_schedule_plan,
};
use xiuxian_wendao_server::transport::{
    ANALYSIS_DOCUMENT_EXTRACT_ROUTE, WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
    WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER, WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
    WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER, WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_UTF8_HEX_HEADER, WENDAO_SCHEMA_VERSION_HEADER,
    encode_document_extract_source_path_utf8_hex,
};

use super::model_route::DocumentExtractModelRoute;
use super::{
    DEFAULT_DOCUMENT_EXTRACT_ENDPOINT, DOCUMENT_EXTRACT_ENDPOINT_ENV,
    DOCUMENT_EXTRACT_ENDPOINTS_ENV, DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES,
    StudioDocumentExtractFlightRouteProvider,
};

impl StudioDocumentExtractFlightRouteProvider {
    pub(super) async fn acquire_document_extract_dispatch_permit(
        &self,
    ) -> Result<OwnedSemaphorePermit, String> {
        let recommended_workers = self.document_extract_recommended_workers();
        let permits = Arc::clone(&self.runtime.conversion_permits);
        if recommended_workers > 0 {
            match permits.try_acquire_owned() {
                Ok(permit) => return Ok(permit),
                Err(TryAcquireError::NoPermits) => {}
                Err(TryAcquireError::Closed) => {
                    return Err("document extract conversion semaphore closed".to_string());
                }
            }
        }

        Arc::clone(&self.runtime.conversion_permits)
            .acquire_owned()
            .await
            .map_err(|error| format!("acquire document extract conversion permit: {error}"))
    }

    pub(super) fn document_extract_recommended_workers(&self) -> u32 {
        let available_permits = self.runtime.conversion_permits.available_permits();
        let active_in_flight = self
            .runtime
            .conversion_limit
            .saturating_sub(available_permits);
        let pressure = document_extract_pressure_evidence(DocumentExtractPressureEvidenceInput {
            max_in_flight: Some(saturating_u32(self.runtime.conversion_limit)),
            active_in_flight: saturating_u32(active_in_flight),
            queued_items: 0,
            failed_items: 0,
            retryable_failures: 0,
            fallback_available: false,
        });
        document_extract_schedule_plan(pressure, Some(1), Some(1), 1).recommended_workers
    }

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

    async fn remove_document_extract_endpoint_channel(&self, endpoint_url: &str) {
        self.runtime.channels.lock().await.remove(endpoint_url);
    }

    pub(super) async fn request_python_document_extract(
        &self,
        source_path: &str,
        output_dir: &str,
        force: bool,
        error_row: bool,
        profile: &str,
    ) -> Result<Vec<EngineRecordBatch>, String> {
        self.request_python_document_extract_with_page_range(
            source_path,
            output_dir,
            force,
            error_row,
            profile,
            None,
        )
        .await
    }

    pub(super) async fn request_python_document_extract_with_model_route(
        &self,
        source_path: &str,
        output_dir: &str,
        force: bool,
        error_row: bool,
        profile: &str,
        model_route: Option<&DocumentExtractModelRoute>,
    ) -> Result<Vec<EngineRecordBatch>, String> {
        self.request_python_document_extract_with_page_range_and_model_route(
            PythonDocumentExtractRequest {
                source_path,
                output_dir,
                force,
                error_row,
                profile,
                page_range: None,
                model_route,
            },
        )
        .await
    }

    pub(super) async fn request_python_document_extract_with_page_range(
        &self,
        source_path: &str,
        output_dir: &str,
        force: bool,
        error_row: bool,
        profile: &str,
        page_range: Option<(u32, u32)>,
    ) -> Result<Vec<EngineRecordBatch>, String> {
        self.request_python_document_extract_with_page_range_and_model_route(
            PythonDocumentExtractRequest {
                source_path,
                output_dir,
                force,
                error_row,
                profile,
                page_range,
                model_route: None,
            },
        )
        .await
    }

    async fn request_python_document_extract_with_page_range_and_model_route(
        &self,
        request: PythonDocumentExtractRequest<'_>,
    ) -> Result<Vec<EngineRecordBatch>, String> {
        let endpoint_urls = self.document_extract_endpoint_attempt_order()?;
        let mut last_retryable_error = None;
        for (attempt_index, endpoint_url) in endpoint_urls.iter().enumerate() {
            match self
                .request_python_document_extract_with_page_range_at_endpoint(
                    PythonDocumentExtractEndpointRequest {
                        endpoint_url,
                        source_path: request.source_path,
                        output_dir: request.output_dir,
                        force: request.force,
                        error_row: request.error_row,
                        profile: request.profile,
                        page_range: request.page_range,
                        model_route: request.model_route,
                    },
                )
                .await
            {
                Ok(batches) => return Ok(batches),
                Err(error)
                    if attempt_index + 1 < endpoint_urls.len()
                        && is_retryable_document_extract_endpoint_error(error.as_str()) =>
                {
                    self.remove_document_extract_endpoint_channel(endpoint_url.as_str())
                        .await;
                    log::warn!(
                        "document extract endpoint `{endpoint_url}` failed with a retryable transport error; trying another endpoint: {error}"
                    );
                    last_retryable_error = Some(error);
                }
                Err(error) => return Err(error),
            }
        }
        Err(last_retryable_error.unwrap_or_else(|| {
            "document extract endpoint pool did not produce a request attempt".to_string()
        }))
    }

    async fn request_python_document_extract_with_page_range_at_endpoint(
        &self,
        request: PythonDocumentExtractEndpointRequest<'_>,
    ) -> Result<Vec<EngineRecordBatch>, String> {
        let channel = self.channel_for_endpoint(request.endpoint_url).await?;

        let inner_client = TonicFlightServiceClient::new(channel)
            .max_encoding_message_size(DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES)
            .max_decoding_message_size(DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES);
        let mut client = FlightClient::new_from_inner(inner_client);
        client
            .add_header(WENDAO_SCHEMA_VERSION_HEADER, "v2")
            .map_err(|error| format!("invalid schema version header: {error}"))?;
        add_source_path_headers(&mut client, request.source_path)?;
        client
            .add_header(
                WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
                request.output_dir,
            )
            .map_err(|error| format!("invalid output dir header: {error}"))?;
        client
            .add_header(
                WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
                if request.force { "true" } else { "false" },
            )
            .map_err(|error| format!("invalid force header: {error}"))?;
        client
            .add_header(
                WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
                if request.error_row { "true" } else { "false" },
            )
            .map_err(|error| format!("invalid error-row header: {error}"))?;
        client
            .add_header(WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER, request.profile)
            .map_err(|error| format!("invalid profile header: {error}"))?;
        if let Some((start, end)) = request.page_range {
            let value = format!("{start}:{end}");
            client
                .add_header(WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER, value.as_str())
                .map_err(|error| format!("invalid page range header: {error}"))?;
        }
        if let Some(model_route) = request.model_route {
            for (key, value) in
                wendao_model_route_metadata(&model_route.intent, &model_route.decision)
            {
                client
                    .add_header(key, value.as_str())
                    .map_err(|error| format!("invalid model route header `{key}`: {error}"))?;
            }
        }

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

    fn document_extract_endpoint_attempt_order(&self) -> Result<Vec<String>, String> {
        let default_endpoint = document_extract_default_endpoint_with_lookup(
            self.configured_default_endpoint.as_deref(),
            &|key| std::env::var(key).ok(),
        );
        let endpoint_urls = document_extract_endpoint_urls(default_endpoint.as_str());
        let request_index = self
            .runtime
            .endpoint_round_robin
            .fetch_add(1, Ordering::Relaxed);
        document_extract_endpoint_attempt_order_for_request(request_index, endpoint_urls.as_slice())
    }
}

struct PythonDocumentExtractRequest<'a> {
    source_path: &'a str,
    output_dir: &'a str,
    force: bool,
    error_row: bool,
    profile: &'a str,
    page_range: Option<(u32, u32)>,
    model_route: Option<&'a DocumentExtractModelRoute>,
}

struct PythonDocumentExtractEndpointRequest<'a> {
    endpoint_url: &'a str,
    source_path: &'a str,
    output_dir: &'a str,
    force: bool,
    error_row: bool,
    profile: &'a str,
    page_range: Option<(u32, u32)>,
    model_route: Option<&'a DocumentExtractModelRoute>,
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

pub(super) fn document_extract_endpoint_attempt_order_for_request(
    request_index: usize,
    endpoint_urls: &[String],
) -> Result<Vec<String>, String> {
    let start_index = endpoint_index_for_request(request_index, endpoint_urls.len())?;
    let mut ordered = Vec::with_capacity(endpoint_urls.len());
    for offset in 0..endpoint_urls.len() {
        let endpoint_index = (start_index + offset) % endpoint_urls.len();
        ordered.push(endpoint_urls[endpoint_index].clone());
    }
    Ok(ordered)
}

pub(super) fn is_retryable_document_extract_endpoint_error(error: &str) -> bool {
    error.contains("failed to connect to document extract endpoint")
        || error.contains("The service is currently unavailable")
        || error.contains("tcp connect error")
        || error.contains("Connection refused")
}

fn add_source_path_headers(client: &mut FlightClient, source_path: &str) -> Result<(), String> {
    let encoded = encode_document_extract_source_path_utf8_hex(source_path);
    client
        .add_header(
            WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_UTF8_HEX_HEADER,
            encoded.as_str(),
        )
        .map_err(|error| format!("invalid encoded source path header: {error}"))?;
    if source_path.is_ascii() {
        client
            .add_header(WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER, source_path)
            .map_err(|error| format!("invalid source path header: {error}"))?;
    }
    Ok(())
}

fn normalize_endpoint(endpoint: &str) -> Option<String> {
    let endpoint = endpoint.trim().trim_end_matches('/');
    (!endpoint.is_empty()).then(|| endpoint.to_string())
}

fn saturating_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
