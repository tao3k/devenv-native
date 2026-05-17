use crate::audio::{AudioShardStrategy, build_audio_shard_plan, planner_input};

#[test]
fn audio_shard_planner_builds_uniform_offsets_in_rust() -> Result<(), String> {
    let plan = build_audio_shard_plan(&planner_input())?;

    assert_eq!(plan.start_offsets_ms, vec![10_000, 140_000, 270_000]);
    assert_eq!(plan.strategy, "uniform");
    assert_eq!(plan.context_before_ms, 2_000);
    assert_eq!(plan.context_after_ms, 3_000);
    Ok(())
}

#[test]
fn audio_shard_planner_builds_head_offsets_in_rust() -> Result<(), String> {
    let mut input = planner_input();
    input.strategy = AudioShardStrategy::Head;

    let plan = build_audio_shard_plan(&input)?;

    assert_eq!(plan.start_offsets_ms, vec![10_000, 40_000, 70_000]);
    assert_eq!(plan.strategy, "head");
    Ok(())
}
