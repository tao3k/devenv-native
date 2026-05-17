use super::{
    AudioShardResult, AudioTranscriptOrgLedgerOptions, build_audio_transcript_org_ledger,
    project_audio_transcript_org_evidence, sample_audio_input,
};

#[cfg(feature = "audio-shard-arrow")]
use super::{build_audio_org_evidence_segment_batch, build_audio_org_evidence_source_batch};

#[cfg(feature = "audio-shard-arrow")]
use arrow::array::{Array, StringArray};

#[test]
fn audio_org_evidence_projection_projects_generated_ledger() -> Result<(), String> {
    let mut first = sample_audio_input("first", "000000.000000000000");
    first.shard_path = "/tmp/audio_000000_first.wav".to_owned();
    let mut second = sample_audio_input("second", "000001.000000030000");
    second.shard_path = "/tmp/audio_000001_second.wav".to_owned();
    second.start_ms = 30_000;
    let results = vec![
        AudioShardResult::succeeded(&first, "First neutral transcript segment.", 0.91),
        AudioShardResult::succeeded(&second, "Second neutral transcript segment.", 0.92),
    ];
    let ledger = build_audio_transcript_org_ledger(
        &[first, second],
        &results,
        &AudioTranscriptOrgLedgerOptions::new("Synthetic transcript", "audio_shards"),
    )?;

    let projection = project_audio_transcript_org_evidence(&ledger)?;

    assert_eq!(projection.source.ledger_kind, "audio_transcript_ledger");
    assert_eq!(projection.source.source_sha256, "sourcehash");
    assert_eq!(projection.source.segment_count, 2);
    assert!(
        projection
            .source
            .evidence_source_id
            .starts_with("audio-org-source:")
    );
    assert_eq!(projection.segments[0].shard_element_id, "first");
    assert_eq!(projection.segments[0].result_element_id.len(), 64);
    assert_eq!(projection.segments[0].chunk_index, 0);
    assert_eq!(projection.segments[0].start_ms, 0);
    assert_eq!(projection.segments[0].duration_ms, 30_000);
    assert_eq!(projection.segments[0].end_ms, 30_000);
    assert_eq!(
        projection.segments[0].source_sha256,
        projection.source.source_sha256
    );
    assert_eq!(
        projection.segments[0].transcript_text,
        "First neutral transcript segment."
    );
    assert!(
        !projection.segments[0]
            .transcript_text
            .contains("[[attachment:")
    );
    assert!(projection.segments[0].transcript_sha256.len() == 64);
    assert_eq!(projection.segments[1].shard_element_id, "second");
    Ok(())
}

#[cfg(feature = "audio-shard-arrow")]
#[test]
fn audio_org_evidence_projection_builds_arrow_batches() -> Result<(), String> {
    let first = sample_audio_input("first", "000000.000000000000");
    let result = AudioShardResult::succeeded(&first, "Neutral transcript segment.", 0.91);
    let ledger = build_audio_transcript_org_ledger(
        std::slice::from_ref(&first),
        &[result],
        &AudioTranscriptOrgLedgerOptions::default(),
    )?;
    let projection = project_audio_transcript_org_evidence(&ledger)?;

    let source_batch =
        build_audio_org_evidence_source_batch(std::slice::from_ref(&projection.source))?;
    let segment_batch = build_audio_org_evidence_segment_batch(&projection.segments)?;

    assert_eq!(source_batch.num_rows(), 1);
    assert_eq!(segment_batch.num_rows(), 1);
    assert_eq!(source_batch.schema().field(0).name(), "contractVersion");
    assert_eq!(segment_batch.schema().field(20).name(), "transcriptText");
    let text_column = segment_batch
        .column_by_name("transcriptText")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| "missing transcriptText column".to_owned())?;
    assert!(!text_column.is_null(0));
    assert_eq!(text_column.value(0), "Neutral transcript segment.");
    Ok(())
}

#[test]
fn audio_org_evidence_projection_rejects_non_audio_ledger() -> Result<(), String> {
    let ledger = "* Notes\n:PROPERTIES:\n:WENDAO_KIND: general_note\n:END:\n\ntext\n";

    let Err(error) = project_audio_transcript_org_evidence(ledger) else {
        return Err("non-audio ledger should fail".to_owned());
    };

    assert!(error.contains("audio_transcript_ledger root"));
    Ok(())
}

#[test]
fn audio_org_evidence_projection_rejects_source_hash_mismatch() -> Result<(), String> {
    let input = sample_audio_input("first", "000000.000000000000");
    let result = AudioShardResult::succeeded(&input, "Neutral transcript segment.", 0.91);
    let ledger = build_audio_transcript_org_ledger(
        std::slice::from_ref(&input),
        &[result],
        &AudioTranscriptOrgLedgerOptions::default(),
    )?
    .replacen(
        ":WENDAO_SOURCE_SHA256: sourcehash",
        ":WENDAO_SOURCE_SHA256: otherhash",
        1,
    );

    let Err(error) = project_audio_transcript_org_evidence(&ledger) else {
        return Err("source hash mismatch should fail".to_owned());
    };

    assert!(error.contains("source hash does not match ledger root"));
    Ok(())
}

#[test]
fn audio_org_evidence_projection_rejects_empty_transcript() -> Result<(), String> {
    let input = sample_audio_input("first", "000000.000000000000");
    let result = AudioShardResult::succeeded(&input, "Neutral transcript segment.", 0.91);
    let ledger = build_audio_transcript_org_ledger(
        std::slice::from_ref(&input),
        &[result],
        &AudioTranscriptOrgLedgerOptions::default(),
    )?
    .replace("Neutral transcript segment.", "");

    let Err(error) = project_audio_transcript_org_evidence(&ledger) else {
        return Err("empty transcript should fail".to_owned());
    };

    assert!(error.contains("empty transcript text"));
    Ok(())
}
