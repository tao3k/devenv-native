use xiuxian_wendao_attachments::audio::AudioShardMergeReport;

use crate::studio::router::handlers::analysis::document_extract::provider::audio::build_audio_transcript_batch;

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
