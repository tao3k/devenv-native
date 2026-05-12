//! `search::queries::graphql::payload` owns Wendao queries graphql payload behavior.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Stable GraphQL-style payload returned by the first query adapter slice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphqlQueryPayload {
    /// Root `data` object keyed by GraphQL field response key.
    pub data: Map<String, Value>,
}
