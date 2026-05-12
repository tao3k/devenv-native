//! `search::queries::flightsql::discovery` owns Wendao queries flightsql discovery behavior.

mod catalogs;
mod schemas;
mod tables;

pub(super) use self::catalogs::{build_catalogs_batch, build_catalogs_flight_info_schema};
pub(super) use self::schemas::{build_schemas_batch, build_schemas_flight_info_schema};
pub(super) use self::tables::{build_tables_batch, build_tables_flight_info_schema};
