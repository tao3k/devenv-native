//! `link_graph::index::search::plan::payload::quantum` owns Wendao plan payload quantum behavior.

#[cfg(feature = "vector-store")]
mod flow;
#[cfg(not(feature = "vector-store"))]
#[path = "flow_disabled.rs"]
mod flow;
#[cfg(feature = "vector-store")]
mod rerank;

#[cfg(all(test, feature = "julia", feature = "vector-store"))]
#[path = "../../../../../../../tests/unit/link_graph/index/search/plan/payload/quantum/mod.rs"]
mod tests;
