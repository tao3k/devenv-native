use std::path::Path;

use xiuxian_wendao_attachments::audio::{
    AudioShardInput, AudioShardManifestItem, AudioShardResult, AudioShardWorkerProfile,
    AudioTranscriptAdmissionOptions, audio_transcript_admission_key,
    audio_transcript_admission_path, lookup_audio_transcript_admission,
    lookup_planned_audio_transcript_admission, persist_audio_transcript_admission,
};

#[test]
fn transcript_admission_roundtrips_successful_rows() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let input = sample_input();
    let options = sample_options(tempdir.path(), "qwen/qwen3-asr-flash-2026-02-10");
    let result = AudioShardResult::succeeded(&input, "transcript text", 0.98);

    let persist_stats = persist_audio_transcript_admission(
        std::slice::from_ref(&input),
        std::slice::from_ref(&result),
        &options,
    )?;
    assert_eq!(persist_stats.stored_count, 1);

    let lookup = lookup_audio_transcript_admission(std::slice::from_ref(&input), &options)?;
    assert_eq!(lookup.stats.hit_count, 1);
    assert_eq!(lookup.stats.miss_count, 0);
    assert_eq!(
        lookup.admitted_results.get(input.shard_element_id.as_str()),
        Some(&result)
    );
    Ok(())
}

#[test]
fn transcript_admission_misses_when_model_changes() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let input = sample_input();
    let first_options = sample_options(tempdir.path(), "model-a");
    let second_options = sample_options(tempdir.path(), "model-b");
    let result = AudioShardResult::succeeded(&input, "transcript text", 0.98);

    persist_audio_transcript_admission(
        std::slice::from_ref(&input),
        std::slice::from_ref(&result),
        &first_options,
    )?;

    let lookup = lookup_audio_transcript_admission(std::slice::from_ref(&input), &second_options)?;
    assert_eq!(lookup.stats.hit_count, 0);
    assert_eq!(lookup.stats.miss_count, 1);
    assert_eq!(lookup.stats.stale_count, 0);
    assert_eq!(lookup.miss_inputs, vec![input]);
    Ok(())
}

#[test]
fn transcript_admission_does_not_store_failed_rows() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let input = sample_input();
    let options = sample_options(tempdir.path(), "qwen/qwen3-asr-flash-2026-02-10");
    let result = AudioShardResult::failed(&input, "model error");

    let persist_stats = persist_audio_transcript_admission(
        std::slice::from_ref(&input),
        std::slice::from_ref(&result),
        &options,
    )?;
    assert_eq!(persist_stats.stored_count, 0);

    let lookup = lookup_audio_transcript_admission(std::slice::from_ref(&input), &options)?;
    assert_eq!(lookup.stats.hit_count, 0);
    assert_eq!(lookup.stats.miss_count, 1);
    Ok(())
}

#[test]
fn planned_transcript_admission_roundtrips_successful_rows() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let input = sample_input();
    let manifest = sample_manifest();
    let profile = sample_profile();
    let options = sample_options(tempdir.path(), "qwen/qwen3-asr-flash-2026-02-10");
    let result = AudioShardResult::succeeded(&input, "transcript text", 0.98);

    let persist_stats = persist_audio_transcript_admission(
        std::slice::from_ref(&input),
        std::slice::from_ref(&result),
        &options,
    )?;
    assert_eq!(persist_stats.planned_stored_count, 1);

    let lookup = lookup_planned_audio_transcript_admission(
        std::slice::from_ref(&manifest),
        &profile,
        &options,
    )?;
    assert!(lookup.all_hit);
    assert_eq!(lookup.stats.hit_count, 1);
    assert_eq!(lookup.stats.planned_hit_count, 1);
    assert_eq!(lookup.stats.planned_miss_count, 0);
    assert_eq!(lookup.inputs, vec![input]);
    assert_eq!(lookup.results, vec![result]);
    Ok(())
}

#[test]
fn planned_transcript_admission_misses_when_model_changes() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let input = sample_input();
    let manifest = sample_manifest();
    let profile = sample_profile();
    let first_options = sample_options(tempdir.path(), "model-a");
    let second_options = sample_options(tempdir.path(), "model-b");
    let result = AudioShardResult::succeeded(&input, "transcript text", 0.98);

    persist_audio_transcript_admission(
        std::slice::from_ref(&input),
        std::slice::from_ref(&result),
        &first_options,
    )?;

    let lookup = lookup_planned_audio_transcript_admission(
        std::slice::from_ref(&manifest),
        &profile,
        &second_options,
    )?;
    assert!(!lookup.all_hit);
    assert_eq!(lookup.stats.hit_count, 0);
    assert_eq!(lookup.stats.planned_hit_count, 0);
    assert_eq!(lookup.stats.planned_miss_count, 1);
    assert_eq!(lookup.stats.planned_stale_count, 0);
    assert!(lookup.inputs.is_empty());
    assert!(lookup.results.is_empty());
    Ok(())
}

#[test]
fn transcript_admission_reports_stale_corrupt_records() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let input = sample_input();
    let options = sample_options(tempdir.path(), "qwen/qwen3-asr-flash-2026-02-10");
    let cache_key = audio_transcript_admission_key(&input, &options)?;
    let cache_path = audio_transcript_admission_path(tempdir.path(), cache_key.as_str());
    std::fs::create_dir_all(
        cache_path
            .parent()
            .ok_or_else(|| "missing cache parent".to_owned())?,
    )
    .map_err(|error| error.to_string())?;
    std::fs::write(cache_path.as_path(), b"{not-json").map_err(|error| error.to_string())?;

    let lookup = lookup_audio_transcript_admission(std::slice::from_ref(&input), &options)?;
    assert_eq!(lookup.stats.hit_count, 0);
    assert_eq!(lookup.stats.miss_count, 1);
    assert_eq!(lookup.stats.stale_count, 1);
    assert_eq!(lookup.miss_inputs, vec![input]);
    Ok(())
}

fn sample_options(cache_dir: &Path, model: &str) -> AudioTranscriptAdmissionOptions {
    AudioTranscriptAdmissionOptions {
        audio_worker: Some("hosted".to_owned()),
        hosted_provider: Some("openrouter".to_owned()),
        hosted_model: Some(model.to_owned()),
        admission_dir: Some(cache_dir.to_path_buf()),
        ..AudioTranscriptAdmissionOptions::default()
    }
}

fn sample_input() -> AudioShardInput {
    AudioShardInput {
        contract_version: "xiuxian_wendao.audio_shard_input.v1".to_owned(),
        source_path: "/tmp/source.mp3".to_owned(),
        source_content_hash: "source-hash".to_owned(),
        shard_path: "/tmp/shard.wav".to_owned(),
        shard_sha256: "shard-hash".to_owned(),
        shard_profile: "audio-shards-v1".to_owned(),
        task_profile: "transcription".to_owned(),
        backend_profile: "hosted-audio-transcript-v1".to_owned(),
        preferred_languages: Vec::new(),
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
        start_ms: 0,
        duration_ms: 30_000,
        media_start_ms: 0,
        media_duration_ms: 30_000,
        context_before_ms: 0,
        context_after_ms: 0,
        shard_element_id: "shard-0001".to_owned(),
        reading_order_key: "000000.000000000000".to_owned(),
    }
}

fn sample_manifest() -> AudioShardManifestItem {
    AudioShardManifestItem {
        shard_id: "shard-0001".to_owned(),
        source_id: "/tmp/source.mp3".to_owned(),
        source_sha256: "source-hash".to_owned(),
        chunk_index: 0,
        start_ms: 0,
        duration_ms: 30_000,
        media_start_ms: 0,
        media_duration_ms: 30_000,
        context_before_ms: 0,
        context_after_ms: 0,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
        cache_key: "audio-shards-v1:shard-0001".to_owned(),
        reading_order_key: "000000.000000000000".to_owned(),
    }
}

fn sample_profile() -> AudioShardWorkerProfile {
    AudioShardWorkerProfile {
        task_profile: "transcription".to_owned(),
        backend_profile: "hosted-audio-transcript-v1".to_owned(),
        preferred_languages: Vec::new(),
    }
}
