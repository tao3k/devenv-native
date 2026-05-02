//! SQL query route contract boundary for Wendao Flight metadata.

mod headers;
#[cfg(feature = "transport")]
mod validation;

pub use headers::{QUERY_SQL_ROUTE, WENDAO_SQL_QUERY_HEADER};
#[cfg(feature = "transport")]
pub use validation::validate_sql_query_request;
