use xiuxian_wendao_attachments::audio::{AudioShardInput, AudioShardMergeReport, AudioShardResult};

use crate::studio::router::handlers::analysis::document_extract::provider::audio::{
    build_audio_transcript_batch, build_audio_transcript_with_org_batch,
};

use super::string_column;

#[test]
fn audio_transcript_batch_rejects_incomplete_merge() {
    let report = AudioShardMergeReport {
        text: "partial".to_owned(),
        timeline_text: "[00:00.000-00:30.000] partial".to_owned(),
        succeeded_count: 1,
        failed_count: 1,
        skipped_count: 0,
        missing_shard_element_ids: Vec::new(),
        failed_shard_element_ids: vec!["failed".to_owned()],
        skipped_shard_element_ids: Vec::new(),
        duplicate_shard_element_ids: Vec::new(),
    };

    assert!(build_audio_transcript_batch("/tmp/a.mp3", "/tmp/out", &report).is_err());
}

#[test]
fn audio_transcript_batch_builds_resource_row() -> Result<(), String> {
    let report = AudioShardMergeReport {
        text: "transcript".to_owned(),
        timeline_text: "[00:00.000-00:30.000] transcript".to_owned(),
        succeeded_count: 1,
        failed_count: 0,
        skipped_count: 0,
        missing_shard_element_ids: Vec::new(),
        failed_shard_element_ids: Vec::new(),
        skipped_shard_element_ids: Vec::new(),
        duplicate_shard_element_ids: Vec::new(),
    };

    let batch = build_audio_transcript_batch("/tmp/a.mp3", "/tmp/out", &report)?;

    assert_eq!(batch.num_rows(), 1);
    let content = batch
        .column_by_name("content")
        .and_then(|column| column.as_any().downcast_ref::<arrow::array::StringArray>())
        .map(|array| array.value(0))
        .ok_or_else(|| "audio transcript resource content should be a string".to_owned())?;
    assert_eq!(content, "[00:00.000-00:30.000] transcript");
    assert_eq!(
        string_column(&batch, "resourceType")?.value(0),
        "audio-transcript"
    );
    Ok(())
}

#[test]
fn audio_transcript_with_org_batch_builds_parallel_ledger_row() -> Result<(), String> {
    let input = sample_audio_input();
    let result = AudioShardResult::succeeded(&input, "transcript", 0.92);
    let report = AudioShardMergeReport {
        text: "transcript".to_owned(),
        timeline_text: "[00:00.000-00:30.000] transcript".to_owned(),
        succeeded_count: 1,
        failed_count: 0,
        skipped_count: 0,
        missing_shard_element_ids: Vec::new(),
        failed_shard_element_ids: Vec::new(),
        skipped_shard_element_ids: Vec::new(),
        duplicate_shard_element_ids: Vec::new(),
    };

    let batch = build_audio_transcript_with_org_batch(
        "/tmp/a.mp3",
        "/tmp/out",
        &report,
        std::slice::from_ref(&input),
        &[result],
    )?;

    assert_eq!(batch.num_rows(), 2);
    assert_eq!(
        string_column(&batch, "resourceType")?.value(0),
        "audio-transcript"
    );
    assert_eq!(
        string_column(&batch, "resourceType")?.value(1),
        "audio-transcript-ledger"
    );
    assert_eq!(string_column(&batch, "mimeType")?.value(0), "text/plain");
    assert_eq!(string_column(&batch, "mimeType")?.value(1), "text/org");
    assert_eq!(
        string_column(&batch, "elementId")?.value(1),
        "_audio_transcript_org"
    );
    assert!(
        string_column(&batch, "content")?
            .value(1)
            .contains("[[attachment:sample.wav][normalized audio shard]]")
    );
    Ok(())
}

fn sample_audio_input() -> AudioShardInput {
    AudioShardInput {
        contract_version: "xiuxian_wendao.audio_shard_input.v1".to_owned(),
        source_path: "/tmp/a.mp3".to_owned(),
        source_content_hash: "sourcehash".to_owned(),
        shard_path: "/tmp/out/audio_shards/sample.wav".to_owned(),
        shard_sha256: "shardhash".to_owned(),
        shard_profile: "audio-shards-v1".to_owned(),
        task_profile: "transcription".to_owned(),
        backend_profile: "hosted-audio-transcript-v1".to_owned(),
        preferred_languages: vec!["auto".to_owned()],
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
        start_ms: 0,
        duration_ms: 30_000,
        media_start_ms: 0,
        media_duration_ms: 30_000,
        context_before_ms: 0,
        context_after_ms: 0,
        shard_element_id: "sample".to_owned(),
        reading_order_key: "000000.000000000000".to_owned(),
    }
}
