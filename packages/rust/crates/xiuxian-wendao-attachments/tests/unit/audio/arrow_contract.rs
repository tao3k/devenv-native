use std::{collections::HashMap, sync::Arc};

use arrow::record_batch::RecordBatch;

use super::{
    AudioShardMaterializationSource, AudioShardMaterializedItem, AudioShardResult,
    AudioShardWorkerProfile, build_audio_shard_input_batch, build_audio_shard_inputs,
    build_audio_shard_result_batch, decode_audio_shard_result_batches, plan_audio_shards,
    sample_plan,
};

#[test]
fn audio_shard_arrow_contract_roundtrips_results() -> Result<(), String> {
    let manifest = plan_audio_shards(&sample_plan())?
        .into_iter()
        .next()
        .ok_or_else(|| "expected one audio shard manifest".to_owned())?;
    let materialized = AudioShardMaterializedItem {
        manifest,
        output_path: std::path::PathBuf::from("/tmp/audio.wav"),
        shard_sha256: "b".repeat(64),
        shard_byte_len: 42,
        materialization_source: AudioShardMaterializationSource::MediaSplitter,
    };
    let profile = AudioShardWorkerProfile::transcription("hosted-audio");

    let inputs = build_audio_shard_inputs(&[materialized], &profile);
    let input_batch = build_audio_shard_input_batch(inputs.as_slice())?;
    let success = AudioShardResult::succeeded(&inputs[0], "transcript", 0.88);
    let result_batch = build_audio_shard_result_batch(std::slice::from_ref(&success))?;
    let decoded = decode_audio_shard_result_batches(&[result_batch])?;

    assert_eq!(input_batch.num_rows(), 1);
    assert_eq!(
        inputs[0].contract_version,
        "xiuxian_wendao.audio_shard_input.v1"
    );
    assert_eq!(inputs[0].task_profile, "transcription");
    assert_eq!(inputs[0].backend_profile, "hosted-audio");
    assert_eq!(decoded, vec![success]);
    Ok(())
}

#[test]
fn audio_shard_arrow_contract_decodes_metadata_free_worker_results() -> Result<(), String> {
    let manifest = plan_audio_shards(&sample_plan())?
        .into_iter()
        .next()
        .ok_or_else(|| "expected one audio shard manifest".to_owned())?;
    let materialized = AudioShardMaterializedItem {
        manifest,
        output_path: std::path::PathBuf::from("/tmp/audio.wav"),
        shard_sha256: "b".repeat(64),
        shard_byte_len: 42,
        materialization_source: AudioShardMaterializationSource::MediaSplitter,
    };
    let profile = AudioShardWorkerProfile::transcription("hosted-audio");
    let inputs = build_audio_shard_inputs(&[materialized], &profile);
    let success = AudioShardResult::succeeded(&inputs[0], "transcript", 0.88);
    let result_batch = build_audio_shard_result_batch(std::slice::from_ref(&success))?;
    let result_batch = without_schema_metadata(&result_batch)?;

    let decoded = decode_audio_shard_result_batches(&[result_batch])?;

    assert_eq!(decoded, vec![success]);
    Ok(())
}

fn without_schema_metadata(batch: &RecordBatch) -> Result<RecordBatch, String> {
    let schema = Arc::new(
        batch
            .schema()
            .as_ref()
            .clone()
            .with_metadata(HashMap::new()),
    );
    RecordBatch::try_new(schema, batch.columns().to_vec())
        .map_err(|error| format!("rebuild audio result batch without metadata: {error}"))
}
