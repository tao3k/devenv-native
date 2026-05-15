//! Cargo entry point for `xiuxian-wendao-server` unit tests.

#[cfg(feature = "transport")]
#[path = "unit/dataset_ontology.rs"]
mod dataset_ontology;
#[path = "unit/lib_policy.rs"]
mod lib_policy;
#[path = "unit/namespace.rs"]
mod namespace;
#[cfg(feature = "transport")]
#[path = "unit/semantic_scope.rs"]
mod semantic_scope;
