//! `search::queries::rest` owns Wendao search queries rest behavior.

mod execution;
mod request;
mod response;

pub use self::execution::query_rest_payload;
pub use self::request::RestQueryRequest;
pub use self::response::RestQueryPayload;

#[cfg(test)]
#[path = "../../../../tests/unit/search/queries/rest/mod.rs"]
mod tests;
