use std::time::Duration;

use arrow::record_batch::RecordBatch;
use arrow_flight::decode::FlightRecordBatchStream;
use arrow_flight::flight_service_client::FlightServiceClient;
use arrow_flight::{FlightDescriptor, Ticket};
use futures::TryStreamExt;
use tonic::Request;
use tonic::metadata::MetadataMap;
use tonic::transport::Endpoint;
use xiuxian_wendao_runtime::transport::flight_descriptor_path;

use super::config::SearchStrategyFlowFlightMaterializationConfig;
use super::rows::row_count;

#[derive(Clone)]
pub(super) struct SearchStrategyFlowFlightClient {
    client: FlightServiceClient<tonic::transport::Channel>,
}

impl SearchStrategyFlowFlightClient {
    pub(super) async fn connect(
        config: &SearchStrategyFlowFlightMaterializationConfig,
    ) -> Result<Self, String> {
        let endpoint = Endpoint::from_shared(config.base_url.clone())
            .map_err(|error| format!("create SearchStrategyFlow Flight endpoint: {error}"))?
            .timeout(Duration::from_secs(config.timeout_seconds));
        let channel = endpoint
            .connect()
            .await
            .map_err(|error| format!("connect SearchStrategyFlow Flight endpoint: {error}"))?;
        Ok(Self {
            client: FlightServiceClient::new(channel),
        })
    }

    pub(super) async fn collect_route_batches<F>(
        &mut self,
        route: &str,
        context: &str,
        populate: F,
    ) -> Result<Vec<RecordBatch>, String>
    where
        F: Fn(&mut MetadataMap) -> Result<(), String>,
    {
        self.collect_route_batches_with_row_policy(route, context, true, populate)
            .await
    }

    pub(super) async fn collect_route_batches_allow_empty<F>(
        &mut self,
        route: &str,
        context: &str,
        populate: F,
    ) -> Result<Vec<RecordBatch>, String>
    where
        F: Fn(&mut MetadataMap) -> Result<(), String>,
    {
        self.collect_route_batches_with_row_policy(route, context, false, populate)
            .await
    }

    async fn collect_route_batches_with_row_policy<F>(
        &mut self,
        route: &str,
        context: &str,
        require_rows: bool,
        populate: F,
    ) -> Result<Vec<RecordBatch>, String>
    where
        F: Fn(&mut MetadataMap) -> Result<(), String>,
    {
        let descriptor_path = flight_descriptor_path(route)
            .map_err(|error| format!("{context} descriptor path: {error}"))?;
        let mut info_request = Request::new(FlightDescriptor::new_path(descriptor_path));
        populate(info_request.metadata_mut())?;
        let flight_info = self
            .client
            .get_flight_info(info_request)
            .await
            .map_err(|error| format!("{context} get_flight_info failed: {error}"))?
            .into_inner();
        let ticket = flight_info
            .endpoint
            .first()
            .and_then(|endpoint| endpoint.ticket.clone())
            .ok_or_else(|| format!("{context} did not return a Flight ticket"))?;
        let mut get_request = Request::new(Ticket {
            ticket: ticket.ticket,
        });
        populate(get_request.metadata_mut())?;
        let response = self
            .client
            .do_get(get_request)
            .await
            .map_err(|error| format!("{context} do_get failed: {error}"))?
            .into_inner()
            .map_err(arrow_flight::error::FlightError::from);
        let mut batch_stream = FlightRecordBatchStream::new_from_flight_data(response);
        let mut batches = Vec::new();
        while let Some(batch) = batch_stream
            .try_next()
            .await
            .map_err(|error| format!("{context} Arrow decode failed: {error}"))?
        {
            batches.push(batch);
        }
        if require_rows && row_count(&batches) == 0 {
            return Err(format!("{context} returned zero decoded rows"));
        }
        Ok(batches)
    }
}
