//! `link_graph::context_snapshot` owns Wendao link graph context snapshot behavior.

#[path = "id.rs"]
pub(crate) mod id;
#[path = "runtime.rs"]
pub(crate) mod runtime;
#[path = "store.rs"]
pub(crate) mod store;
#[cfg(test)]
#[path = "../../../tests/unit/link_graph/context_snapshot/mod.rs"]
mod tests;
#[path = "types.rs"]
pub(crate) mod types;

pub use id::quantum_context_snapshot_id;
pub use store::{
    valkey_quantum_context_snapshot_drop, valkey_quantum_context_snapshot_get,
    valkey_quantum_context_snapshot_get_with_valkey, valkey_quantum_context_snapshot_rollback,
    valkey_quantum_context_snapshot_rollback_with_valkey, valkey_quantum_context_snapshot_save,
    valkey_quantum_context_snapshot_save_with_valkey,
};
pub use types::{LINK_GRAPH_QUANTUM_CONTEXT_SNAPSHOT_SCHEMA_VERSION, QuantumContextSnapshot};
