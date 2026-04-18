#[path = "kernel/adjacency.rs"]
pub(crate) mod adjacency;
#[path = "kernel/iteration.rs"]
pub(crate) mod iteration;
#[path = "kernel/runtime.rs"]
pub(crate) mod runtime;
#[cfg(test)]
#[path = "../../../../tests/unit/link_graph/index/ppr/kernel/mod.rs"]
mod tests;
#[path = "kernel/types.rs"]
pub(crate) mod types;
