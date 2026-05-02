#[cfg(feature = "search-runtime")]
#[path = "core/mod.rs"]
mod core;
/// Shared `FlightSQL` adapter surface over the request-scoped query system.
#[cfg(feature = "search-runtime")]
#[path = "flightsql/mod.rs"]
pub mod flightsql;
/// Shared `GraphQL` adapter surface over the request-scoped query system.
#[cfg(feature = "search-runtime")]
#[path = "graphql/mod.rs"]
pub mod graphql;
/// Shared `REST` adapter surface over the request-scoped query system.
#[cfg(feature = "search-runtime")]
#[path = "rest/mod.rs"]
pub mod rest;
/// Shared `SQL` adapter surface over the request-scoped query system.
#[cfg(feature = "search-runtime")]
#[path = "sql/mod.rs"]
pub mod sql;

#[cfg(feature = "search-runtime")]
pub use self::core::SearchQueryService;

#[cfg(all(test, feature = "search-runtime"))]
#[path = "../../../tests/unit/search/queries/mod.rs"]
mod tests;
