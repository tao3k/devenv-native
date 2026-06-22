//! Cargo entry point for xiuxian-zhenfa integration tests.

#[cfg(all(feature = "client", feature = "gateway"))]
#[path = "integration/client.rs"]
mod client;
#[path = "integration/context_extensions.rs"]
mod context_extensions;
#[cfg(feature = "contract-validation")]
#[path = "integration/contract_validation.rs"]
mod contract_validation;
#[path = "integration/contracts.rs"]
mod contracts;
#[path = "integration/error_mapping.rs"]
mod error_mapping;
#[path = "integration/transmuter.rs"]
mod transmuter;
#[path = "integration/xml_lite.rs"]
mod xml_lite;
