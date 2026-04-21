use std::sync::Arc;

use arrow::record_batch::RecordBatch as EngineRecordBatch;
use arrow_flight::client::FlightClient;
use arrow_flight::FlightDescriptor;
use async_trait::async_trait;
use futures::TryStreamExt;
use tonic::transport::Endpoint;
use xiuxian_wendao_runtime::transport::{
    AnalysisFlightRouteResponse, PdfExtractFlightRouteProvider,
};
use xiuxian_vector_store::engine_batches_to_lance_batches;

use crate::gateway::studio::router::GatewayState;

const DEFAULT_PDF_EXTRACT_ENDPOINT: &str = "http://localhost:50051";

#[derive(Clone)]
pub(crate) struct StudioPdfExtractFlightRouteProvider {
    #[allow(dead_code)]
    state: Arc<GatewayState>,
}

impl StudioPdfExtractFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self { state }
    }
}

impl std::fmt::Debug for StudioPdfExtractFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioPdfExtractFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl PdfExtractFlightRouteProvider for StudioPdfExtractFlightRouteProvider {
    async fn pdf_extract_batch(
        &self,
        source_path: &str,
        output_dir: &str,
        extract_images: bool,
        extract_tables: bool,
        extract_formulas: bool,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        let endpoint_url = std::env::var("WENDAO_PDF_EXTRACT_ENDPOINT")
            .unwrap_or_else(|_| DEFAULT_PDF_EXTRACT_ENDPOINT.to_string());

        let endpoint = Endpoint::from_shared(endpoint_url.clone()).map_err(|error| {
            format!("invalid PDF extract endpoint `{endpoint_url}`: {error}")
        })?;

        let channel = endpoint.connect().await.map_err(|error| {
            format!("failed to connect to PDF extract endpoint `{endpoint_url}`: {error}")
        })?;

        let mut client = FlightClient::new(channel);
        client
            .add_header("x-wendao-schema-version", "v2")
            .map_err(|error| format!("invalid schema version header: {error}"))?;
        client
            .add_header("x-wendao-pdf-extract-source-path", source_path)
            .map_err(|error| format!("invalid source path header: {error}"))?;
        client
            .add_header("x-wendao-pdf-extract-output-dir", output_dir)
            .map_err(|error| format!("invalid output dir header: {error}"))?;
        client
            .add_header(
                "x-wendao-pdf-extract-images",
                if extract_images { "true" } else { "false" },
            )
            .map_err(|error| format!("invalid images header: {error}"))?;
        client
            .add_header(
                "x-wendao-pdf-extract-tables",
                if extract_tables { "true" } else { "false" },
            )
            .map_err(|error| format!("invalid tables header: {error}"))?;
        client
            .add_header(
                "x-wendao-pdf-extract-formulas",
                if extract_formulas { "true" } else { "false" },
            )
            .map_err(|error| format!("invalid formulas header: {error}"))?;

        let descriptor = FlightDescriptor::new_path(vec!["analysis".to_string(), "pdf-extract".to_string()]);
        let flight_info = client
            .get_flight_info(descriptor)
            .await
            .map_err(|error| format!("PDF extract get_flight_info failed: {error}"))?;

        let ticket = flight_info
            .endpoint
            .first()
            .and_then(|endpoint| endpoint.ticket.clone())
            .ok_or_else(|| "PDF extract flight info missing ticket".to_string())?;

        let stream = client
            .do_get(ticket)
            .await
            .map_err(|error| format!("PDF extract do_get failed: {error}"))?;

        let engine_batches: Vec<EngineRecordBatch> = stream
            .try_collect()
            .await
            .map_err(|error| format!("PDF extract stream decode failed: {error}"))?;

        if engine_batches.is_empty() {
            return Err("PDF extract returned no record batches".to_string());
        }

        let lance_batches = engine_batches_to_lance_batches(&engine_batches)
            .map_err(|error| format!("PDF extract batch conversion failed: {error}"))?;

        let batch = lance_batches
            .into_iter()
            .next()
            .ok_or_else(|| "PDF extract produced no Lance batches".to_string())?;

        Ok(AnalysisFlightRouteResponse::new(batch))
    }
}
