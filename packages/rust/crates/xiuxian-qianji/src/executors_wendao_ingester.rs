//! Native `Wendao` ingestion mechanism for memory-promotion workflows.

#[path = "executors/wendao_ingester/entity.rs"]
mod entity;
#[path = "executors_wendao_ingester_mechanism.rs"]
mod mechanism;
#[path = "executors/wendao_ingester/persistence.rs"]
mod persistence;
#[path = "executors/wendao_ingester/scope.rs"]
mod scope;

pub use mechanism::WendaoIngesterMechanism;
