//! `link_graph::index::ppr::kernel` owns Wendao index ppr kernel behavior.

#[path = "adjacency.rs"]
pub(crate) mod adjacency;
#[path = "iteration.rs"]
pub(crate) mod iteration;
#[path = "runtime.rs"]
pub(crate) mod runtime;
#[cfg(test)]
#[path = "../../../../../tests/unit/link_graph/index/ppr/kernel/mod.rs"]
mod tests;
#[path = "types.rs"]
pub(crate) mod types;
