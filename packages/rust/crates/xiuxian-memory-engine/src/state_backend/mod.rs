//! Memory state persistence backends.

mod facade;
#[cfg(feature = "valkey")]
mod valkey;

pub use facade::{
    LocalMemoryStateStore, MemoryStateStore, default_valkey_recall_feedback_hash_key,
    default_valkey_state_hash_keys, default_valkey_state_key,
};
#[cfg(feature = "valkey")]
pub use valkey::ValkeyMemoryStateStore;
