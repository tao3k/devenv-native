#[path = "discovery/mod.rs"]
mod discovery;
mod execution;
mod metadata;
mod service;
mod statement;

pub use self::service::{StudioFlightSqlService, build_studio_flightsql_service};

#[cfg(test)]
#[path = "../../../../tests/unit/search/queries/flightsql/mod.rs"]
mod tests;
