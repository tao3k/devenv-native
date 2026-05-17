use super::{
    AudioShardMaterializedItem, AudioShardResult, AudioShardWorkerProfile,
    build_audio_shard_input_batch, build_audio_shard_inputs, build_audio_shard_result_batch,
    decode_audio_shard_result_batches, plan_audio_shards, sample_plan,
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
