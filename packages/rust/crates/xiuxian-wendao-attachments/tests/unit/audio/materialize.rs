use super::{
    AudioShardMaterializationInput, AudioShardMaterializationSource, error_to_string,
    make_executable, materialize_audio_shards, plan_audio_shards, sample_plan,
};

#[test]
fn audio_materialization_runs_splitter_with_planned_media_windows() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(error_to_string)?;
    let source_path = tempdir.path().join("source.mp3");
    std::fs::write(source_path.as_path(), b"source").map_err(error_to_string)?;
    let ffmpeg_path = tempdir.path().join("fake_ffmpeg.sh");
    let log_path = tempdir.path().join("ffmpeg.log");
    std::fs::write(
        ffmpeg_path.as_path(),
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> '{}'\nlast=''\nfor arg in \"$@\"; do last=\"$arg\"; done\ntouch \"$last\"\n",
            log_path.display()
        ),
    )
    .map_err(error_to_string)?;
    make_executable(ffmpeg_path.as_path())?;
    let mut plan = sample_plan();
    plan.start_offsets_ms = vec![30_000];
    plan.context_before_ms = 2_000;
    plan.context_after_ms = 3_000;
    let input = AudioShardMaterializationInput {
        source_path: source_path.clone(),
        output_dir: tempdir.path().join("chunks"),
        ffmpeg_path,
        artifact_cache_dir: None,
        force: true,
    };

    let items = materialize_audio_shards(&plan, &input)?;

    assert_eq!(items.len(), 1);
    assert!(items[0].output_path.exists());
    assert_eq!(items[0].manifest.media_start_ms, 28_000);
    assert_eq!(items[0].manifest.media_duration_ms, 35_000);
    let log = std::fs::read_to_string(log_path).map_err(error_to_string)?;
    assert!(log.contains("-ss"));
    assert!(log.contains("28.000"));
    assert!(log.contains("-t"));
    assert!(log.contains("35.000"));
    assert!(log.contains(source_path.to_string_lossy().as_ref()));
    Ok(())
}

#[test]
fn audio_materialization_reuses_existing_chunks_without_splitter() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(error_to_string)?;
    let source_path = tempdir.path().join("source.mp3");
    std::fs::write(source_path.as_path(), b"source").map_err(error_to_string)?;
    let ffmpeg_path = tempdir.path().join("missing_ffmpeg");
    let output_dir = tempdir.path().join("chunks");
    std::fs::create_dir_all(output_dir.as_path()).map_err(error_to_string)?;
    let mut plan = sample_plan();
    plan.start_offsets_ms = vec![0];
    let existing_manifest = plan_audio_shards(&plan)?
        .into_iter()
        .next()
        .ok_or_else(|| "expected one audio shard manifest".to_owned())?;
    let existing_path = output_dir.join(format!(
        "audio_{:06}_{}.{}",
        existing_manifest.chunk_index,
        existing_manifest
            .shard_id
            .chars()
            .take(16)
            .collect::<String>(),
        existing_manifest.audio_format
    ));
    std::fs::write(existing_path.as_path(), b"cached").map_err(error_to_string)?;
    let input = AudioShardMaterializationInput {
        source_path,
        output_dir,
        ffmpeg_path,
        artifact_cache_dir: None,
        force: false,
    };

    let items = materialize_audio_shards(&plan, &input)?;

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].output_path, existing_path);
    Ok(())
}

#[cfg(feature = "foyer-artifact-cache")]
#[test]
fn audio_materialization_restores_missing_output_from_artifact_cache() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(error_to_string)?;
    let source_path = tempdir.path().join("source.mp3");
    std::fs::write(source_path.as_path(), b"source").map_err(error_to_string)?;
    let ffmpeg_path = tempdir.path().join("fake_ffmpeg.sh");
    std::fs::write(
        ffmpeg_path.as_path(),
        "#!/bin/sh\nlast=''\nfor arg in \"$@\"; do last=\"$arg\"; done\nprintf cached > \"$last\"\n",
    )
    .map_err(error_to_string)?;
    make_executable(ffmpeg_path.as_path())?;
    let mut plan = sample_plan();
    plan.start_offsets_ms = vec![0];
    let cache_dir = tempdir.path().join("artifact-cache");
    let first_input = AudioShardMaterializationInput {
        source_path: source_path.clone(),
        output_dir: tempdir.path().join("first"),
        ffmpeg_path,
        artifact_cache_dir: Some(cache_dir.clone()),
        force: false,
    };
    let first_items = materialize_audio_shards(&plan, &first_input)?;
    assert_eq!(first_items.len(), 1);

    let second_input = AudioShardMaterializationInput {
        source_path,
        output_dir: tempdir.path().join("second"),
        ffmpeg_path: tempdir.path().join("missing_ffmpeg"),
        artifact_cache_dir: Some(cache_dir),
        force: false,
    };
    let second_items = materialize_audio_shards(&plan, &second_input)?;

    assert_eq!(second_items.len(), 1);
    assert_eq!(
        std::fs::read(second_items[0].output_path.as_path()).map_err(error_to_string)?,
        b"cached"
    );
    assert_eq!(second_items[0].shard_byte_len, 6);
    assert_eq!(
        second_items[0].materialization_source,
        AudioShardMaterializationSource::ArtifactCache
    );
    assert_eq!(second_items[0].shard_sha256, first_items[0].shard_sha256);
    Ok(())
}

#[cfg(feature = "foyer-artifact-cache")]
#[test]
fn audio_materialization_force_restores_verified_artifact_cache() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(error_to_string)?;
    let source_path = tempdir.path().join("source.mp3");
    std::fs::write(source_path.as_path(), b"source").map_err(error_to_string)?;
    let ffmpeg_path = tempdir.path().join("fake_ffmpeg.sh");
    let log_path = tempdir.path().join("ffmpeg.log");
    std::fs::write(
        ffmpeg_path.as_path(),
        format!(
            "#!/bin/sh\nprintf 'split\\n' >> '{}'\nlast=''\nfor arg in \"$@\"; do last=\"$arg\"; done\nprintf cached > \"$last\"\n",
            log_path.display()
        ),
    )
    .map_err(error_to_string)?;
    make_executable(ffmpeg_path.as_path())?;
    let mut plan = sample_plan();
    plan.start_offsets_ms = vec![0];
    let cache_dir = tempdir.path().join("artifact-cache");
    let first_input = AudioShardMaterializationInput {
        source_path: source_path.clone(),
        output_dir: tempdir.path().join("first"),
        ffmpeg_path,
        artifact_cache_dir: Some(cache_dir.clone()),
        force: true,
    };
    let first_items = materialize_audio_shards(&plan, &first_input)?;
    assert_eq!(first_items.len(), 1);
    assert_eq!(
        first_items[0].materialization_source,
        AudioShardMaterializationSource::MediaSplitter
    );

    let second_input = AudioShardMaterializationInput {
        source_path,
        output_dir: tempdir.path().join("second"),
        ffmpeg_path: tempdir.path().join("missing_ffmpeg"),
        artifact_cache_dir: Some(cache_dir),
        force: true,
    };
    let second_items = materialize_audio_shards(&plan, &second_input)?;

    assert_eq!(second_items.len(), 1);
    assert_eq!(
        second_items[0].materialization_source,
        AudioShardMaterializationSource::ArtifactCache
    );
    assert_eq!(
        std::fs::read(second_items[0].output_path.as_path()).map_err(error_to_string)?,
        b"cached"
    );
    assert_eq!(second_items[0].shard_byte_len, 6);
    assert_eq!(second_items[0].shard_sha256, first_items[0].shard_sha256);
    assert_eq!(
        std::fs::read_to_string(log_path).map_err(error_to_string)?,
        "split\n"
    );
    Ok(())
}
