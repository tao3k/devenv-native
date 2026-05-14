//! Result cache identity builders for backend-neutral `audio` shards.

use super::identity::sha256_hex;
use super::types::AudioResultCacheInput;

/// Build a deterministic cache key for a downstream audio task result.
///
/// # Errors
///
/// Returns an error when any identity field is empty.
pub fn audio_result_cache_key(input: &AudioResultCacheInput) -> Result<String, String> {
    validate_result_cache_input(input)?;
    Ok(format!(
        "{}:{}:{}",
        input.task_profile.trim(),
        input.backend_id.trim(),
        sha256_hex(
            format!(
                "{}:{}:{}:{}",
                input.shard_cache_key.trim(),
                input.task_profile.trim(),
                input.backend_id.trim(),
                input.backend_config_hash.trim()
            )
            .as_bytes()
        )
    ))
}

fn validate_result_cache_input(input: &AudioResultCacheInput) -> Result<(), String> {
    if input.shard_cache_key.trim().is_empty() {
        return Err("audio result shard cache key cannot be empty".to_owned());
    }
    if input.task_profile.trim().is_empty() {
        return Err("audio result task profile cannot be empty".to_owned());
    }
    if input.backend_id.trim().is_empty() {
        return Err("audio result backend id cannot be empty".to_owned());
    }
    if input.backend_config_hash.trim().is_empty() {
        return Err("audio result backend config hash cannot be empty".to_owned());
    }
    Ok(())
}
