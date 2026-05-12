//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
#[path = "contracts.rs"]
mod contracts;

pub use contracts::{
    API_HEALTH_AXUM_PATH, API_HEALTH_OPENAPI_PATH, API_NOTIFY_AXUM_PATH, API_NOTIFY_OPENAPI_PATH,
    API_STATS_AXUM_PATH, API_STATS_OPENAPI_PATH,
};
