//! SQL query metadata header definitions for Wendao Flight routes.

/// Canonical SQL query text metadata header for Wendao Flight requests.
pub const WENDAO_SQL_QUERY_HEADER: &str = "x-wendao-sql-query";
/// Stable route for the read-only SQL query contract.
pub const QUERY_SQL_ROUTE: &str = "/query/sql";
