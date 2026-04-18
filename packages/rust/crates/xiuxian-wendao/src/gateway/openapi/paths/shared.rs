#[path = "shared/contracts.rs"]
mod contracts;
#[path = "shared/inventory/mod.rs"]
pub(crate) mod inventory;

#[cfg(test)]
#[path = "../../../../tests/unit/gateway/openapi/paths/shared/mod.rs"]
mod tests;

pub use contracts::{
    API_HEALTH_AXUM_PATH, API_HEALTH_OPENAPI_PATH, API_NOTIFY_AXUM_PATH, API_NOTIFY_OPENAPI_PATH,
    API_STATS_AXUM_PATH, API_STATS_OPENAPI_PATH, RouteContract,
};
pub use inventory::WENDAO_GATEWAY_ROUTE_CONTRACTS;
