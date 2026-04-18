#[cfg(feature = "search-runtime")]
#[path = "queries/core/mod.rs"]
mod core;
/// Shared `FlightSQL` adapter surface over the request-scoped query system.
#[cfg(feature = "search-runtime")]
#[path = "queries/flightsql/mod.rs"]
pub mod flightsql;
/// Shared `GraphQL` adapter surface over the request-scoped query system.
#[cfg(feature = "search-runtime")]
#[path = "queries/graphql.rs"]
pub mod graphql;
/// Shared `REST` adapter surface over the request-scoped query system.
#[cfg(feature = "search-runtime")]
#[path = "queries/rest/mod.rs"]
pub mod rest;
/// Shared `SQL` adapter surface over the request-scoped query system.
#[cfg(feature = "search-runtime")]
#[path = "queries/sql.rs"]
pub mod sql;

#[cfg(feature = "search-runtime")]
pub use self::core::SearchQueryService;

#[cfg(all(test, feature = "search-runtime"))]
#[path = "../../tests/unit/search/queries/mod.rs"]
mod tests;
