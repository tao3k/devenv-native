use xiuxian_config_core::{resolve_data_home, resolve_project_root_or_cwd};

pub(super) fn default_memory_path() -> String {
    let project_root = resolve_project_root_or_cwd();
    let data_home = resolve_data_home(Some(project_root.as_path()))
        .unwrap_or_else(|| project_root.join(".data"));

    data_home
        .join("xiuxian-daochang")
        .join("memory")
        .to_string_lossy()
        .to_string()
}

pub(super) fn default_embedding_dim() -> usize {
    384
}

pub(super) fn default_memory_table() -> String {
    "episodes".to_string()
}

pub(super) fn default_recall_k1() -> usize {
    20
}

pub(super) fn default_recall_k2() -> usize {
    5
}

pub(super) fn default_recall_lambda() -> f32 {
    0.3
}

pub(super) fn default_memory_persistence_backend() -> String {
    "auto".to_string()
}

pub(super) fn default_memory_persistence_key_prefix() -> String {
    "xiuxian-daochang:memory".to_string()
}

pub(super) fn default_recall_credit_enabled() -> bool {
    true
}

pub(super) fn default_recall_credit_max_candidates() -> usize {
    4
}

pub(super) fn default_decay_enabled() -> bool {
    true
}

pub(super) fn default_decay_every_turns() -> usize {
    24
}

pub(super) fn default_decay_factor() -> f32 {
    0.985
}

pub(super) fn default_gate_promote_threshold() -> f32 {
    0.78
}

pub(super) fn default_gate_obsolete_threshold() -> f32 {
    0.32
}

pub(super) fn default_gate_promote_min_usage() -> u32 {
    3
}

pub(super) fn default_gate_obsolete_min_usage() -> u32 {
    2
}

pub(super) fn default_gate_promote_failure_rate_ceiling() -> f32 {
    0.25
}

pub(super) fn default_gate_obsolete_failure_rate_floor() -> f32 {
    0.70
}

pub(super) fn default_gate_promote_min_ttl_score() -> f32 {
    0.50
}

pub(super) fn default_gate_obsolete_max_ttl_score() -> f32 {
    0.45
}

pub(super) fn default_stream_consumer_enabled() -> bool {
    true
}

pub(super) fn default_stream_name() -> String {
    "memory.events".to_string()
}

pub(super) fn default_stream_consumer_group() -> String {
    "xiuxian-daochang-memory".to_string()
}

pub(super) fn default_stream_consumer_name_prefix() -> String {
    "agent".to_string()
}

pub(super) fn default_stream_consumer_batch_size() -> usize {
    32
}

pub(super) fn default_stream_consumer_block_ms() -> u64 {
    1000
}
