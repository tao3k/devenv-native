//! SearchStrategyFlow decoded materialization receipt support.

mod fixture;
mod flight;
mod receipt;

pub use fixture::materialize_fixture_receipt_json;
pub use receipt::{
    RouteDecodedPayloadReceipt, RouteMaterializationReceipt,
    SearchStrategyFlowMaterializationError, SearchStrategyFlowMaterializationReceipt,
};
