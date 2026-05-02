use std::collections::BTreeSet;

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use arrow_flight::sql::CommandGetDbSchemas;
use tonic::Status;

use crate::search::queries::sql::SqlQuerySurface;

use super::catalogs::WENDAO_FLIGHTSQL_CATALOG_NAME;

pub(in crate::search::queries::flightsql) fn build_schemas_flight_info_schema(
    query: CommandGetDbSchemas,
) -> SchemaRef {
    query.into_builder().schema()
}

pub(in crate::search::queries::flightsql) fn build_schemas_batch(
    query: CommandGetDbSchemas,
    surface: &SqlQuerySurface,
) -> Result<RecordBatch, Status> {
    let mut builder = query.into_builder();
    for schema_name in discovery_schema_names(surface) {
        builder.append(WENDAO_FLIGHTSQL_CATALOG_NAME, schema_name);
    }
    builder.build().map_err(|error| {
        Status::internal(format!(
            "FlightSQL failed to build schemas discovery batch: {error}"
        ))
    })
}

pub(in crate::search::queries::flightsql) fn flightsql_schema_name(scope: &str) -> &str {
    match scope {
        "local" | "local_logical" => "local",
        "repo" | "repo_logical" => "repo",
        "system" => "system",
        other => other,
    }
}

fn discovery_schema_names(surface: &SqlQuerySurface) -> BTreeSet<&str> {
    surface
        .tables
        .iter()
        .map(|table| flightsql_schema_name(table.scope.as_str()))
        .collect()
}
