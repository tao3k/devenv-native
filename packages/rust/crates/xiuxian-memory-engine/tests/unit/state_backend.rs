//! State backend key-derivation tests for `xiuxian-memory-engine`.

use xiuxian_memory_engine::{
    StoreConfig, default_valkey_recall_feedback_hash_key, default_valkey_state_hash_keys,
    default_valkey_state_key,
};

fn store_config(path: &str, table_name: &str) -> StoreConfig {
    StoreConfig {
        path: path.to_string(),
        embedding_dim: 384,
        table_name: table_name.to_string(),
    }
}

#[test]
fn default_valkey_state_key_is_deterministic_for_same_store_config() {
    let config = store_config("/tmp/xiuxian-memory-engine", "episodes");

    let key_a = default_valkey_state_key("lingchong-bot:memory", &config);
    let key_b = default_valkey_state_key("lingchong-bot:memory", &config);

    assert_eq!(key_a, key_b);
}

#[test]
fn default_valkey_state_key_changes_with_store_identity() {
    let base = store_config("/tmp/xiuxian-memory-engine", "episodes");
    let changed_path = store_config("/tmp/xiuxian-memory-engine-other", "episodes");
    let changed_table = store_config("/tmp/xiuxian-memory-engine", "episodes_v2");

    let base_key = default_valkey_state_key("lingchong-bot:memory", &base);
    let path_key = default_valkey_state_key("lingchong-bot:memory", &changed_path);
    let table_key = default_valkey_state_key("lingchong-bot:memory", &changed_table);

    assert_ne!(base_key, path_key);
    assert_ne!(base_key, table_key);
}

#[test]
fn default_valkey_state_hash_keys_are_deterministic() {
    let config = store_config("/tmp/xiuxian-memory-engine", "episodes");
    let base_key = default_valkey_state_key("lingchong-bot:memory", &config);

    let hash_keys_a = default_valkey_state_hash_keys(&base_key);
    let hash_keys_b = default_valkey_state_hash_keys(&base_key);

    assert_eq!(hash_keys_a, hash_keys_b);
    assert!(hash_keys_a.episodes.ends_with(":episodes"));
    assert!(hash_keys_a.q_values.ends_with(":q_values"));
}

#[test]
fn default_valkey_recall_feedback_hash_key_is_deterministic() {
    let config = store_config("/tmp/xiuxian-memory-engine", "episodes");
    let base_key = default_valkey_state_key("lingchong-bot:memory", &config);
    let key_a = default_valkey_recall_feedback_hash_key(&base_key);
    let key_b = default_valkey_recall_feedback_hash_key(&base_key);

    assert_eq!(key_a, key_b);
    assert!(key_a.ends_with(":recall_feedback"));
}
