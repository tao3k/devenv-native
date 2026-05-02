//! `FlightService` implementation for the runtime-owned Wendao service core.

use arrow_flight::decode::FlightRecordBatchStream;
use arrow_flight::encode::FlightDataEncoderBuilder;
use arrow_flight::flight_service_server::FlightService;
use arrow_flight::{
    Action, Criteria, Empty, FlightData, FlightDescriptor, FlightEndpoint, FlightInfo,
    HandshakeRequest, PollInfo, SchemaResult, Ticket,
};
use async_trait::async_trait;
use futures::stream;
use futures::{StreamExt, TryStreamExt};
use tonic::{Request, Response, Status};
use xiuxian_db_store::{EngineRecordBatch, lance_batch_to_engine_batch};

use crate::transport::query_contract::{
    validate_rerank_request_batch, validate_rerank_response_batch,
};

use crate::transport::server::{
    ActionResultStream, ActionTypeStream, FlightDataStream, FlightInfoStream, HandshakeStream,
    PutResultStream, descriptor_route, ticket_route, validate_rerank_dimension_header,
    validate_rerank_min_final_score_header, validate_rerank_top_k_header, validate_schema_version,
};

use super::ServiceCore as WendaoFlightService;

const ANALYSIS_ROUTE_PREFIX: &str = "/analysis/";

#[async_trait]
impl FlightService for WendaoFlightService {
    type HandshakeStream = HandshakeStream;
    type ListFlightsStream = FlightInfoStream;
    type DoGetStream = FlightDataStream;
    type DoPutStream = PutResultStream;
    type DoExchangeStream = FlightDataStream;
    type DoActionStream = ActionResultStream;
    type ListActionsStream = ActionTypeStream;

    async fn handshake(
        &self,
        _request: Request<tonic::Streaming<HandshakeRequest>>,
    ) -> Result<Response<Self::HandshakeStream>, Status> {
        Err(Status::unimplemented(
            "handshake is not used by this Flight service",
        ))
    }

    async fn list_flights(
        &self,
        _request: Request<Criteria>,
    ) -> Result<Response<Self::ListFlightsStream>, Status> {
        Err(Status::unimplemented(
            "list_flights is not used by this Flight service",
        ))
    }

    async fn get_flight_info(
        &self,
        request: Request<FlightDescriptor>,
    ) -> Result<Response<FlightInfo>, Status> {
        validate_schema_version(request.metadata(), self.expected_schema_version.as_str())?;
        let metadata = request.metadata().clone();
        let descriptor = request.into_inner();
        let route = descriptor_route(&descriptor)?;
        let cache_key = Self::route_request_cache_key(route.as_str(), &metadata)?;
        let route_payload = self
            .cached_route_payload(route.as_str(), &metadata, &cache_key)
            .await?;
        if route.starts_with(ANALYSIS_ROUTE_PREFIX) {
            let route_payload = route_payload.clone();
            drop(tokio::spawn(async move {
                let _ = route_payload.do_get_frames().await;
            }));
        }
        let endpoint = FlightEndpoint::new().with_ticket(Ticket::new(route.clone()));
        let schema = route_payload.schema();
        let flight_info = FlightInfo::new()
            .try_with_schema(schema.as_ref())
            .map_err(|error| Status::internal(error.to_string()))?
            .with_endpoint(endpoint)
            .with_descriptor(descriptor)
            .with_total_records(route_payload.total_rows()?)
            .with_app_metadata(route_payload.app_metadata.clone());
        Ok(Response::new(flight_info))
    }

    async fn poll_flight_info(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<PollInfo>, Status> {
        Err(Status::unimplemented(
            "poll_flight_info is not used by this Flight service",
        ))
    }

    async fn get_schema(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<SchemaResult>, Status> {
        Err(Status::unimplemented(
            "get_schema is not used by this Flight service",
        ))
    }

    async fn do_get(
        &self,
        request: Request<Ticket>,
    ) -> Result<Response<Self::DoGetStream>, Status> {
        validate_schema_version(request.metadata(), self.expected_schema_version.as_str())?;
        let metadata = request.metadata().clone();
        let ticket = request.into_inner();
        let route = ticket_route(&ticket)?;
        let cache_key = Self::route_request_cache_key(route.as_str(), &metadata)?;
        let do_get_frames = self
            .cached_route_payload(route.as_str(), &metadata, &cache_key)
            .await?
            .do_get_frames()
            .await?;
        let response_stream =
            stream::unfold((do_get_frames, 0_usize), |(frames, index)| async move {
                frames
                    .get(index)
                    .cloned()
                    .map(|frame| (Ok::<FlightData, Status>(frame), (frames, index + 1)))
            });
        Ok(Response::new(Box::pin(response_stream)))
    }

    async fn do_put(
        &self,
        _request: Request<tonic::Streaming<FlightData>>,
    ) -> Result<Response<Self::DoPutStream>, Status> {
        Err(Status::unimplemented(
            "do_put is not used by this Flight service",
        ))
    }

    async fn do_exchange(
        &self,
        request: Request<tonic::Streaming<FlightData>>,
    ) -> Result<Response<Self::DoExchangeStream>, Status> {
        validate_schema_version(request.metadata(), self.expected_schema_version.as_str())?;
        let expected_dimension = validate_rerank_dimension_header(request.metadata())?;
        let top_k = validate_rerank_top_k_header(request.metadata())?;
        let min_final_score = validate_rerank_min_final_score_header(request.metadata())?;

        let stream = request
            .into_inner()
            .map(|frame| frame.map_err(arrow_flight::error::FlightError::from))
            .try_filter(|frame| futures::future::ready(!frame.data_header.is_empty()));
        let mut batch_stream = FlightRecordBatchStream::new_from_flight_data(stream);
        let mut request_batches = Vec::new();
        while let Some(batch) = batch_stream.try_next().await.map_err(|error| {
            Status::invalid_argument(format!("failed to decode rerank request batch: {error}"))
        })? {
            validate_rerank_request_batch(&batch, expected_dimension)
                .map_err(Status::invalid_argument)?;
            request_batches.push(batch);
        }
        if request_batches.is_empty() {
            return Err(Status::invalid_argument(
                "expected at least one rerank record batch",
            ));
        }

        let exchange_response_batch = self
            .rerank_handler
            .handle_exchange_batches(&request_batches, top_k, min_final_score)
            .map_err(Status::invalid_argument)?;
        let exchange_response_batch = lance_batch_to_engine_batch(&exchange_response_batch)
            .map_err(|error| Status::internal(error.to_string()))?;
        validate_rerank_response_batch(&exchange_response_batch).map_err(Status::internal)?;

        let response_stream = FlightDataEncoderBuilder::new()
            .build(stream::iter(vec![Ok::<
                EngineRecordBatch,
                arrow_flight::error::FlightError,
            >(exchange_response_batch)]))
            .map(|item| item.map_err(|error| Status::internal(error.to_string())));
        Ok(Response::new(Box::pin(response_stream)))
    }

    async fn do_action(
        &self,
        _request: Request<Action>,
    ) -> Result<Response<Self::DoActionStream>, Status> {
        Err(Status::unimplemented(
            "do_action is not used by this Flight service",
        ))
    }

    async fn list_actions(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::ListActionsStream>, Status> {
        Err(Status::unimplemented(
            "list_actions is not used by this Flight service",
        ))
    }
}
