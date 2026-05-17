use super::{
    AudioRiskParentSelectionOptions, AudioShardRequestMetric, AudioShardResult, sample_audio_input,
    select_audio_risk_parent_shards,
};

#[test]
fn audio_risk_parent_selection_uses_rust_text_and_latency_facts() -> Result<(), String> {
    let mut first = sample_audio_input("first", "000000.000000000000");
    first.duration_ms = 60_000;
    let mut second = sample_audio_input("second", "000001.000000060000");
    second.start_ms = 60_000;
    second.duration_ms = 60_000;
    let mut third = sample_audio_input("third", "000002.000000120000");
    third.start_ms = 120_000;
    third.duration_ms = 60_000;
    let inputs = vec![second.clone(), third.clone(), first.clone()];
    let results = vec![
        AudioShardResult::succeeded(&first, "开场介绍", 0.9),
        AudioShardResult::succeeded(&second, "重复重复重复重复重复重复通用测试会议", 0.9),
        AudioShardResult::succeeded(&third, "结束总结", 0.9),
    ];
    let request_metrics = vec![AudioShardRequestMetric {
        shard_element_id: "second".to_owned(),
        wall_ms: 60_000,
    }];

    let selected = select_audio_risk_parent_shards(
        inputs.as_slice(),
        results.as_slice(),
        request_metrics.as_slice(),
        AudioRiskParentSelectionOptions::default(),
    )?;

    assert_eq!(
        selected
            .iter()
            .map(|row| row.shard_element_id.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "second", "third"]
    );
    assert_eq!(
        selected[0].reasons,
        vec!["low-text-density", "timeline-boundary"]
    );
    assert!(selected[1].reasons.contains(&"high-repetition".to_owned()));
    assert!(selected[1].reasons.contains(&"high-latency".to_owned()));
    assert_eq!(
        selected[2].reasons,
        vec!["low-text-density", "timeline-boundary"]
    );
    Ok(())
}

#[test]
fn audio_risk_parent_selection_reserves_boundaries_under_limit() -> Result<(), String> {
    let mut first = sample_audio_input("first", "000000.000000000000");
    first.duration_ms = 60_000;
    let mut second = sample_audio_input("second", "000001.000000060000");
    second.start_ms = 60_000;
    second.duration_ms = 60_000;
    let mut third = sample_audio_input("third", "000002.000000120000");
    third.start_ms = 120_000;
    third.duration_ms = 60_000;
    let inputs = vec![first.clone(), second.clone(), third.clone()];
    let results = vec![
        AudioShardResult::succeeded(&first, "开场介绍", 0.9),
        AudioShardResult::succeeded(&second, "重复重复重复重复重复重复通用测试会议", 0.9),
        AudioShardResult::succeeded(&third, "结束总结", 0.9),
    ];
    let mut options = xiuxian_wendao_attachments::audio::AudioRiskParentSelectionOptions {
        limit_parents: 2,
        ..Default::default()
    };

    let selected = select_audio_risk_parent_shards(&inputs, &results, &[], options)?;

    assert_eq!(
        selected
            .iter()
            .map(|row| row.shard_element_id.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "third"]
    );

    options.include_boundaries = false;
    options.max_chars_per_minute = 0.0;
    options.max_chinese_ratio = 0.0;
    let selected_without_boundaries =
        select_audio_risk_parent_shards(&inputs, &results, &[], options)?;
    assert_eq!(
        selected_without_boundaries
            .iter()
            .map(|row| row.shard_element_id.as_str())
            .collect::<Vec<_>>(),
        vec!["second"]
    );
    Ok(())
}

#[test]
fn audio_risk_parent_selection_includes_failed_rows_for_recovery() -> Result<(), String> {
    let mut first = sample_audio_input("first", "000000.000000000000");
    first.duration_ms = 60_000;
    let mut second = sample_audio_input("second", "000001.000000060000");
    second.start_ms = 60_000;
    second.duration_ms = 60_000;
    let inputs = vec![first.clone(), second.clone()];
    let results = vec![
        AudioShardResult::failed(&first, "audio transcript quality gate failed"),
        AudioShardResult::skipped(&second, "not configured"),
    ];
    let options = xiuxian_wendao_attachments::audio::AudioRiskParentSelectionOptions {
        include_boundaries: false,
        ..Default::default()
    };

    let selected = select_audio_risk_parent_shards(&inputs, &results, &[], options)?;

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].shard_element_id, "first");
    assert_eq!(selected[0].reasons, vec!["failed-result"]);
    Ok(())
}
