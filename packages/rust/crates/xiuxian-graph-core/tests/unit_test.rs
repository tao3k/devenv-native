//! Unit tests for the shared graph-core crate.

#[path = "unit/projection.rs"]
mod projection;

#[cfg(feature = "mermaid")]
#[path = "unit/mermaid.rs"]
mod mermaid;

#[cfg(feature = "petgraph")]
#[path = "unit/petgraph.rs"]
mod petgraph;
