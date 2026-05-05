//! Zhenfa native-tool branch for bridge setup, hooks, and signal sinks.

mod bridge;
pub(crate) mod valkey_hooks;

pub(crate) use bridge::ZhenfaToolBridge;
pub(crate) use bridge::{
    test_memory_reward_signal_sink, test_memory_reward_signal_sink_with_valkey_backend,
    test_runtime_deps,
};
