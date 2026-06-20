//! Studio HTTP and gateway adapter boundary for Wendao.
//!
//! This crate owns Studio-facing HTTP routes, `OpenAPI` exports, and adapters
//! that connect UI/API requests to Wendao domain services. Low-level Flight and
//! gRPC transport contracts are provided by `xiuxian-wendao-server`.

#![recursion_limit = "256"]

#[cfg(all(test, feature = "document-extract-audio-shards"))]
#[path = "test_support/unit/mod.rs"]
pub(crate) mod test_support_unit;

#[cfg(all(test, feature = "document-extract-audio-shards"))]
pub(crate) use test_support_unit as unit;

/// Lightweight Studio route, schema, and `OpenAPI` contracts.
#[cfg(feature = "contracts")]
pub mod contracts;

/// Stable `OpenAPI` contract exports for the Studio gateway.
#[cfg(feature = "contracts")]
pub mod openapi;

/// Runtime support for the Studio-owned command-line and service binaries.
#[cfg(any(feature = "cli-bin-support", feature = "flight-server-bin-support"))]
#[doc(hidden)]
pub mod bin_support;

/// Flight and gRPC transport facade used by Studio adapters.
#[cfg(feature = "flight-transport")]
pub use xiuxian_wendao_server::transport;

/// Studio HTTP, Flight, and gateway route surfaces.
#[cfg(feature = "studio")]
pub mod studio;
