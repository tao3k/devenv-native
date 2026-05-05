use arrow::datatypes::{Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use arrow_flight::sql::CommandGetTables;
use tonic::Status;

use crate::search::queries::sql::SqlQuerySurface;

use super::catalogs::WENDAO_FLIGHTSQL_CATALOG_NAME;
use super::schemas::flightsql_schema_name;

pub(in crate::search::queries::flightsql) fn build_tables_flight_info_schema(
    query: CommandGetTables,
) -> SchemaRef {
    query.into_builder().schema()
}

pub(in crate::search::queries::flightsql) fn build_tables_batch(
    surface: &SqlQuerySurface,
    query: CommandGetTables,
) -> Result<RecordBatch, Status> {
    let include_schema = query.include_schema;
    let mut builder = query.into_builder();
    let empty_schema = Schema::empty();

    for table in &surface.tables {
        let table_schema = if include_schema {
            surface.arrow_schema_for_table(table.sql_table_name.as_str())
        } else {
            empty_schema.clone()
        };
        builder
            .append(
                WENDAO_FLIGHTSQL_CATALOG_NAME,
                flightsql_schema_name(table.scope.as_str()),
                table.sql_table_name.as_str(),
                flightsql_table_type(table.sql_object_kind.as_str()),
                &table_schema,
            )
            .map_err(|error| {
                Status::internal(format!(
                    "FlightSQL failed to append discovery table `{}`: {error}",
                    table.sql_table_name
                ))
            })?;
    }

    builder.build().map_err(|error| {
        Status::internal(format!(
            "FlightSQL failed to build tables discovery batch: {error}"
        ))
    })
}

pub(in crate::search::queries::flightsql) fn flightsql_table_type(sql_object_kind: &str) -> &str {
    match sql_object_kind {
        "view" => "VIEW",
        "system" => "SYSTEM TABLE",
        _ => "TABLE",
    }
}
