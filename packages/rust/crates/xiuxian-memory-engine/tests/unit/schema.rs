//! Schema validation tests for `EpisodeMetadata`.

use xiuxian_memory_engine::{Episode, EpisodeDraft, EpisodeMetadata};

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_metadata_validation_valid() -> TestResult {
    let mut episode = Episode::new(EpisodeDraft {
        id: ("ep-1".to_string()).into(),
        intent: "intent".to_string(),
        intent_embedding: vec![0.1, 0.2],
        experience: "exp".to_string(),
        outcome: "ok".to_string(),
        scope: None,
    });
    episode.q_value = 0.5;
    episode.retrieval_count = 2;
    episode.created_at = 12_345;
    episode.updated_at = 12_345;
    let m = EpisodeMetadata::from_episode(&episode)?;
    assert!((m.q_value - 0.5).abs() < f32::EPSILON);
    assert_eq!(m.experience, "exp");
    assert_eq!(m.retrieval_count, 2);
    Ok(())
}

#[test]
fn test_metadata_validation_q_out_of_range() {
    let mut episode = Episode::new(EpisodeDraft {
        id: ("ep-1".to_string()).into(),
        intent: "intent".to_string(),
        intent_embedding: vec![0.1, 0.2],
        experience: "a".to_string(),
        outcome: "b".to_string(),
        scope: None,
    });
    episode.q_value = 1.5;
    assert!(EpisodeMetadata::from_episode(&episode).is_err());
    episode.q_value = -0.1;
    assert!(EpisodeMetadata::from_episode(&episode).is_err());
}

#[test]
fn test_metadata_roundtrip() -> TestResult {
    let mut episode = Episode::new(EpisodeDraft {
        id: ("ep-1".to_string()).into(),
        intent: "intent".to_string(),
        intent_embedding: vec![0.1, 0.2],
        experience: "exp".to_string(),
        outcome: "out".to_string(),
        scope: None,
    });
    episode.q_value = 0.7;
    episode.retrieval_count = 3;
    episode.success_count = 1;
    episode.failure_count = 2;
    episode.created_at = 999;
    episode.updated_at = 1000;
    let m = EpisodeMetadata::from_episode(&episode)?;
    let json = m.to_json()?;
    let m2 = EpisodeMetadata::from_json(&json)?;
    assert_eq!(m.experience, m2.experience);
    assert!((m.q_value - m2.q_value).abs() < f32::EPSILON);
    assert_eq!(m.updated_at, m2.updated_at);
    Ok(())
}

#[test]
fn test_metadata_empty_json_fails() {
    assert!(EpisodeMetadata::from_json("").is_err());
    assert!(EpisodeMetadata::from_json("   ").is_err());
}

#[test]
fn test_metadata_invalid_json_fails() {
    assert!(EpisodeMetadata::from_json("{invalid").is_err());
}

#[test]
fn test_metadata_missing_required_fields_fails() {
    assert!(EpisodeMetadata::from_json(r#"{"experience":"x","outcome":"y"}"#).is_err());
    assert!(EpisodeMetadata::from_json("{}").is_err());
}

#[test]
fn test_metadata_invalid_q_value_in_json_fails() {
    let json = r#"{"experience":"x","outcome":"y","q_value":1.5,"retrieval_count":0,"success_count":0,"failure_count":0,"created_at":0,"updated_at":0}"#;
    assert!(EpisodeMetadata::from_json(json).is_err());
}
