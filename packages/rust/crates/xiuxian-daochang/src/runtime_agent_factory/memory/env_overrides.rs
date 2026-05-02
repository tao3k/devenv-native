use super::embedding::apply_memory_env_embedding_overrides;
use crate::MemoryConfig;
use crate::env_parse::{
    parse_bool_from_env, parse_positive_f32_from_env, parse_positive_u32_from_env,
    parse_positive_u64_from_env, parse_positive_usize_from_env, parse_unit_f32_from_env,
};
use crate::runtime_agent_factory::env_lookup::non_empty_env;

pub(super) fn apply_memory_env_overrides(memory: &mut MemoryConfig) {
    apply_memory_env_embedding_overrides(memory);
    apply_memory_env_persistence_overrides(memory);
    apply_memory_env_recall_gate_overrides(memory);
    apply_memory_env_stream_overrides(memory);
}

fn apply_memory_env_persistence_overrides(memory: &mut MemoryConfig) {
    if let Some(backend) = non_empty_env("XIUXIAN_DAOCHANG_MEMORY_PERSISTENCE_BACKEND") {
        memory.persistence_backend = backend;
    }
    if memory.persistence_valkey_url.is_none()
        && let Some(url) = non_empty_env("VALKEY_URL")
    {
        memory.persistence_valkey_url = Some(url);
    }
    if let Some(prefix) = non_empty_env("XIUXIAN_DAOCHANG_MEMORY_VALKEY_KEY_PREFIX") {
        memory.persistence_key_prefix = prefix;
    }
    if let Some(strict_startup) =
        parse_bool_from_env("XIUXIAN_DAOCHANG_MEMORY_PERSISTENCE_STRICT_STARTUP")
    {
        memory.persistence_strict_startup = Some(strict_startup);
    }
}

fn apply_memory_env_recall_gate_overrides(memory: &mut MemoryConfig) {
    if let Some(enabled) = parse_bool_from_env("XIUXIAN_DAOCHANG_MEMORY_RECALL_CREDIT_ENABLED") {
        memory.recall_credit_enabled = enabled;
    }
    if let Some(max_candidates) =
        parse_positive_usize_from_env("XIUXIAN_DAOCHANG_MEMORY_RECALL_CREDIT_MAX_CANDIDATES")
    {
        memory.recall_credit_max_candidates = max_candidates;
    }
    if let Some(enabled) = parse_bool_from_env("XIUXIAN_DAOCHANG_MEMORY_DECAY_ENABLED") {
        memory.decay_enabled = enabled;
    }
    if let Some(every_turns) =
        parse_positive_usize_from_env("XIUXIAN_DAOCHANG_MEMORY_DECAY_EVERY_TURNS")
    {
        memory.decay_every_turns = every_turns;
    }
    if let Some(factor) = parse_positive_f32_from_env("XIUXIAN_DAOCHANG_MEMORY_DECAY_FACTOR") {
        memory.decay_factor = factor;
    }
    if let Some(threshold) =
        parse_unit_f32_from_env("XIUXIAN_DAOCHANG_MEMORY_GATE_PROMOTE_THRESHOLD")
    {
        memory.gate_promote_threshold = threshold;
    }
    if let Some(threshold) =
        parse_unit_f32_from_env("XIUXIAN_DAOCHANG_MEMORY_GATE_OBSOLETE_THRESHOLD")
    {
        memory.gate_obsolete_threshold = threshold;
    }
    if let Some(min_usage) =
        parse_positive_u32_from_env("XIUXIAN_DAOCHANG_MEMORY_GATE_PROMOTE_MIN_USAGE")
    {
        memory.gate_promote_min_usage = min_usage;
    }
    if let Some(min_usage) =
        parse_positive_u32_from_env("XIUXIAN_DAOCHANG_MEMORY_GATE_OBSOLETE_MIN_USAGE")
    {
        memory.gate_obsolete_min_usage = min_usage;
    }
    if let Some(rate) =
        parse_unit_f32_from_env("XIUXIAN_DAOCHANG_MEMORY_GATE_PROMOTE_FAILURE_RATE_CEILING")
    {
        memory.gate_promote_failure_rate_ceiling = rate;
    }
    if let Some(rate) =
        parse_unit_f32_from_env("XIUXIAN_DAOCHANG_MEMORY_GATE_OBSOLETE_FAILURE_RATE_FLOOR")
    {
        memory.gate_obsolete_failure_rate_floor = rate;
    }
    if let Some(score) =
        parse_unit_f32_from_env("XIUXIAN_DAOCHANG_MEMORY_GATE_PROMOTE_MIN_TTL_SCORE")
    {
        memory.gate_promote_min_ttl_score = score;
    }
    if let Some(score) =
        parse_unit_f32_from_env("XIUXIAN_DAOCHANG_MEMORY_GATE_OBSOLETE_MAX_TTL_SCORE")
    {
        memory.gate_obsolete_max_ttl_score = score;
    }
}

fn apply_memory_env_stream_overrides(memory: &mut MemoryConfig) {
    if let Some(enabled) = parse_bool_from_env("XIUXIAN_DAOCHANG_MEMORY_STREAM_CONSUMER_ENABLED") {
        memory.stream_consumer_enabled = enabled;
    }
    if let Some(stream_name) = non_empty_env("XIUXIAN_DAOCHANG_MEMORY_STREAM_NAME") {
        memory.stream_name = stream_name;
    }
    if let Some(group) = non_empty_env("XIUXIAN_DAOCHANG_MEMORY_STREAM_CONSUMER_GROUP") {
        memory.stream_consumer_group = group;
    }
    if let Some(prefix) = non_empty_env("XIUXIAN_DAOCHANG_MEMORY_STREAM_CONSUMER_NAME_PREFIX") {
        memory.stream_consumer_name_prefix = prefix;
    }
    if let Some(batch_size) =
        parse_positive_usize_from_env("XIUXIAN_DAOCHANG_MEMORY_STREAM_CONSUMER_BATCH_SIZE")
    {
        memory.stream_consumer_batch_size = batch_size;
    }
    if let Some(block_ms) =
        parse_positive_u64_from_env("XIUXIAN_DAOCHANG_MEMORY_STREAM_CONSUMER_BLOCK_MS")
    {
        memory.stream_consumer_block_ms = block_ms;
    }
}
