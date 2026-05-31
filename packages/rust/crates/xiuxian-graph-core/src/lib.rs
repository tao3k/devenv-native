//! Shared graph projection primitives and optional graph rendering adapters.
//!
//! This crate owns reusable graph infrastructure only. Domain surfaces such as
//! Org tasks, SDD records, Wendao `LinkGraph`, or Qianji BPMN relations should
//! map their local semantics into these generic projections at their own
//! package boundary.

mod model;

#[cfg(feature = "mermaid")]
mod mermaid;
#[cfg(feature = "petgraph")]
mod petgraph_adapter;

pub use model::{GraphEdge, GraphNode, GraphNodeId, GraphProjection, GraphProjectionError};

#[cfg(feature = "mermaid")]
pub use mermaid::{CompactMermaidGraph, MermaidDirection, MermaidGraphError};
#[cfg(feature = "petgraph")]
pub use petgraph_adapter::{StableGraphProjection, to_stable_di_graph};
