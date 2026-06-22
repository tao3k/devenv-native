//! `search::queries::flightsql::service` owns Wendao queries flightsql service behavior.

use std::sync::{Arc, OnceLock};

use arrow::datatypes::Schema;
use arrow_flight::sql::server::FlightSqlService;
use arrow_flight::sql::{
    CommandGetCatalogs, CommandGetDbSchemas, CommandGetSqlInfo, CommandGetTables,
    CommandStatementQuery, ProstMessageExt,
};
use arrow_flight::{FlightInfo, Ticket};
use prost::Message;
use tonic::{Request, Response, Status};

use crate::search::queries::SearchQueryService;
use crate::search::queries::sql::configured_parquet_query_engine;

use super::discovery::{
    build_catalogs_batch, build_catalogs_flight_info_schema, build_schemas_batch,
    build_schemas_flight_info_schema, build_tables_batch, build_tables_flight_info_schema,
};
use super::execution::execute_flightsql_statement_query;
use super::metadata::STUDIO_FLIGHT_SQL_INFO_DATA;
use super::statement::{
    StatementCache, cache_statement_batches, new_statement_cache, new_statement_handle,
    response_stream, statement_flight_info, take_statement_batches,
};

#[derive(Clone)]
/// Shared-query `FlightSQL` service over the request-scoped SQL surface.
pub struct StudioFlightSqlService {
    query_service: SearchQueryService,
    statement_cache: StatementCache,
    statement_query_engine: Arc<OnceLock<Result<crate::duckdb::ParquetQueryEngine, String>>>,
}

#[must_use]
/// Build a `FlightSQL` service that reuses the shared query system.
pub fn build_studio_flightsql_service(
    query_service: impl Into<SearchQueryService>,
) -> StudioFlightSqlService {
    StudioFlightSqlService::new(query_service)
}

impl StudioFlightSqlService {
    #[must_use]
    /// Create one `FlightSQL` service over the provided search-plane service.
    pub fn new(query_service: impl Into<SearchQueryService>) -> Self {
        Self {
            query_service: query_service.into(),
            statement_cache: new_statement_cache(),
            statement_query_engine: Arc::new(OnceLock::new()),
        }
    }

    fn discovery_flight_info(
        descriptor: arrow_flight::FlightDescriptor,
        ticket: Ticket,
        schema: &Schema,
        error_context: &str,
    ) -> Result<Response<FlightInfo>, Status> {
        let endpoint = arrow_flight::FlightEndpoint::new().with_ticket(ticket);
        FlightInfo::new()
            .try_with_schema(schema)
            .map_err(|error| Status::internal(format!("{error_context}: {error}")))
            .map(|flight_info| {
                Response::new(
                    flight_info
                        .with_endpoint(endpoint)
                        .with_descriptor(descriptor),
                )
            })
    }

    fn statement_query_engine(&self) -> Result<crate::duckdb::ParquetQueryEngine, Status> {
        self.statement_query_engine
            .get_or_init(|| configured_parquet_query_engine(self.query_service.search_plane()))
            .clone()
            .map_err(|error| {
                Status::internal(format!(
                    "FlightSQL failed to configure published parquet query engine: {error}"
                ))
            })
    }
}

#[tonic::async_trait]
impl FlightSqlService for StudioFlightSqlService {
    type FlightService = Self;

    async fn get_flight_info_statement(
        &self,
        query: CommandStatementQuery,
        request: Request<arrow_flight::FlightDescriptor>,
    ) -> Result<Response<FlightInfo>, Status> {
        if query.transaction_id.is_some() {
            return Err(Status::unimplemented(
                "FlightSQL transactions are not implemented in the first Wendao slice",
            ));
        }

        let descriptor = request.into_inner();
        let statement_query_engine = self.statement_query_engine()?;
        let result = execute_flightsql_statement_query(
            &self.query_service,
            Some(&statement_query_engine),
            query.query.as_str(),
        )
        .await
        .map_err(|error| {
            Status::internal(format!("FlightSQL statement execution failed: {error}"))
        })?;
        let _statement_route = &result.route;
        let batches = result.batches;
        let statement_handle = new_statement_handle();
        let flight_info = statement_flight_info(descriptor, statement_handle.as_str(), &batches)?;
        cache_statement_batches(&self.statement_cache, statement_handle, batches);
        Ok(Response::new(flight_info))
    }

    async fn get_flight_info_sql_info(
        &self,
        query: CommandGetSqlInfo,
        request: Request<arrow_flight::FlightDescriptor>,
    ) -> Result<Response<FlightInfo>, Status> {
        let descriptor = request.into_inner();
        let ticket = Ticket::new(query.as_any().encode_to_vec());
        let endpoint = arrow_flight::FlightEndpoint::new().with_ticket(ticket);
        let schema = query.into_builder(&STUDIO_FLIGHT_SQL_INFO_DATA).schema();
        let flight_info = FlightInfo::new()
            .try_with_schema(schema.as_ref())
            .map_err(|error| {
                Status::internal(format!(
                    "FlightSQL failed to encode sql_info schema: {error}"
                ))
            })?
            .with_endpoint(endpoint)
            .with_descriptor(descriptor);
        Ok(Response::new(flight_info))
    }

    async fn get_flight_info_catalogs(
        &self,
        query: CommandGetCatalogs,
        request: Request<arrow_flight::FlightDescriptor>,
    ) -> Result<Response<FlightInfo>, Status> {
        let descriptor = request.into_inner();
        let ticket = Ticket::new(query.as_any().encode_to_vec());
        let schema = build_catalogs_flight_info_schema(query);
        Self::discovery_flight_info(
            descriptor,
            ticket,
            schema.as_ref(),
            "FlightSQL failed to encode catalogs discovery schema",
        )
    }

    async fn get_flight_info_schemas(
        &self,
        query: CommandGetDbSchemas,
        request: Request<arrow_flight::FlightDescriptor>,
    ) -> Result<Response<FlightInfo>, Status> {
        let descriptor = request.into_inner();
        let ticket = Ticket::new(query.as_any().encode_to_vec());
        let schema = build_schemas_flight_info_schema(query);
        Self::discovery_flight_info(
            descriptor,
            ticket,
            schema.as_ref(),
            "FlightSQL failed to encode schemas discovery schema",
        )
    }

    async fn get_flight_info_tables(
        &self,
        query: CommandGetTables,
        request: Request<arrow_flight::FlightDescriptor>,
    ) -> Result<Response<FlightInfo>, Status> {
        let descriptor = request.into_inner();
        let ticket = Ticket::new(query.as_any().encode_to_vec());
        let schema = build_tables_flight_info_schema(query);
        Self::discovery_flight_info(
            descriptor,
            ticket,
            schema.as_ref(),
            "FlightSQL failed to encode tables discovery schema",
        )
    }

    async fn do_get_statement(
        &self,
        ticket: arrow_flight::sql::TicketStatementQuery,
        _request: Request<Ticket>,
    ) -> Result<
        Response<<Self as arrow_flight::flight_service_server::FlightService>::DoGetStream>,
        Status,
    > {
        let batches = take_statement_batches(&self.statement_cache, &ticket)?;
        let schema = batches.first().map_or_else(
            || Arc::new(Schema::empty()),
            xiuxian_db_store::EngineRecordBatch::schema,
        );
        Ok(response_stream(schema, batches))
    }

    async fn do_get_sql_info(
        &self,
        query: CommandGetSqlInfo,
        _request: Request<Ticket>,
    ) -> Result<
        Response<<Self as arrow_flight::flight_service_server::FlightService>::DoGetStream>,
        Status,
    > {
        let builder = query.into_builder(&STUDIO_FLIGHT_SQL_INFO_DATA);
        let schema = builder.schema();
        let batch = builder.build().map_err(|error| {
            Status::internal(format!("FlightSQL failed to build sql_info batch: {error}"))
        })?;
        Ok(response_stream(schema, vec![batch]))
    }

    async fn do_get_catalogs(
        &self,
        query: CommandGetCatalogs,
        _request: Request<Ticket>,
    ) -> Result<
        Response<<Self as arrow_flight::flight_service_server::FlightService>::DoGetStream>,
        Status,
    > {
        let batch = build_catalogs_batch(query)?;
        let schema = batch.schema();
        Ok(response_stream(schema, vec![batch]))
    }

    async fn do_get_schemas(
        &self,
        query: CommandGetDbSchemas,
        _request: Request<Ticket>,
    ) -> Result<
        Response<<Self as arrow_flight::flight_service_server::FlightService>::DoGetStream>,
        Status,
    > {
        let query_surface = self
            .query_service
            .open_sql_surface()
            .await
            .map_err(|error| {
                Status::internal(format!(
                    "FlightSQL failed to build discovery SQL surface: {error}"
                ))
            })?;
        let batch = build_schemas_batch(query, &query_surface)?;
        let schema = batch.schema();
        Ok(response_stream(schema, vec![batch]))
    }

    async fn do_get_tables(
        &self,
        query: CommandGetTables,
        _request: Request<Ticket>,
    ) -> Result<
        Response<<Self as arrow_flight::flight_service_server::FlightService>::DoGetStream>,
        Status,
    > {
        let query_surface = self
            .query_service
            .open_sql_surface()
            .await
            .map_err(|error| {
                Status::internal(format!(
                    "FlightSQL failed to build discovery SQL surface: {error}"
                ))
            })?;
        let batch = build_tables_batch(&query_surface, query)?;
        let schema = batch.schema();
        Ok(response_stream(schema, vec![batch]))
    }

    async fn register_sql_info(&self, _sql_info_key: i32, _result: &arrow_flight::sql::SqlInfo) {}
}
