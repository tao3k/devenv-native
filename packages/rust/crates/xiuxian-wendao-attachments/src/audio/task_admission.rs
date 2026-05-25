//! Task admission identity builders for backend-neutral `audio` shards.

use super::identity::sha256_hex;
use super::types::AudioTaskAdmissionInput;

/// Build a deterministic admission key for a downstream audio task invocation.
///
/// # Errors
///
/// Returns an error when any identity field is empty.
pub fn audio_task_admission_key(input: &AudioTaskAdmissionInput) -> Result<String, String> {
    validate_task_admission_input(input)?;
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

fn validate_task_admission_input(input: &AudioTaskAdmissionInput) -> Result<(), String> {
    if input.shard_cache_key.trim().is_empty() {
        return Err("audio task admission shard identity cannot be empty".to_owned());
    }
    if input.task_profile.trim().is_empty() {
        return Err("audio task admission task profile cannot be empty".to_owned());
    }
    if input.backend_id.trim().is_empty() {
        return Err("audio task admission backend id cannot be empty".to_owned());
    }
    if input.backend_config_hash.trim().is_empty() {
        return Err("audio task admission backend config hash cannot be empty".to_owned());
    }
    Ok(())
}
