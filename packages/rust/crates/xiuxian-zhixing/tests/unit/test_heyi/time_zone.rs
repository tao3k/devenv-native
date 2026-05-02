use super::support::{
    Arc, EchoManifestation, KnowledgeGraph, MarkdownStorage, TestResult, ZhixingHeyi, context,
    tempdir,
};

#[test]
fn test_time_zone_parsing() -> TestResult {
    let context = context("UTC")?;
    assert_eq!(context.heyi.time_zone.to_string(), "UTC");
    Ok(())
}

#[test]
fn test_invalid_time_zone_returns_config_error() -> TestResult {
    let graph = Arc::new(KnowledgeGraph::new());
    let temp_dir = tempdir()?;
    let storage = Arc::new(MarkdownStorage::new(temp_dir.path().to_path_buf()));
    let manifestation = Arc::new(EchoManifestation);

    let result = ZhixingHeyi::new(
        graph,
        manifestation,
        storage,
        "test".to_string(),
        "Invalid/Zone",
    );
    match result {
        Ok(_) => panic!("Expected invalid time-zone constructor to fail"),
        Err(error) => assert!(error.to_string().contains("Invalid time zone")),
    }
    Ok(())
}
