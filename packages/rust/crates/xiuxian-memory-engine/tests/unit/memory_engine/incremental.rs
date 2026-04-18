use super::{Episode, TestResult, test_store};

#[test]
fn test_incremental_update_episode() -> TestResult {
    let store = test_store("test");

    let mut ep = Episode::new(
        "ep-1".to_string(),
        "debug api".to_string(),
        store.encoder().encode("debug api"),
        "Solution v1".to_string(),
        "failure".to_string(),
    );
    ep.created_at = 123;
    ep.updated_at = 123;
    store.store(ep)?;

    let retrieved = store
        .get("ep-1")
        .ok_or_else(|| std::io::Error::other("ep-1 should exist before update"))?;
    assert_eq!(retrieved.experience, "Solution v1");
    assert_eq!(retrieved.outcome, "failure");
    assert_eq!(retrieved.created_at, 123);
    assert_eq!(retrieved.updated_at, 123);

    let updated = store.update_episode("ep-1", "Solution v2 - fixed", "success");
    assert!(updated, "Update should return true");

    let retrieved = store
        .get("ep-1")
        .ok_or_else(|| std::io::Error::other("ep-1 should exist after update"))?;
    assert_eq!(retrieved.experience, "Solution v2 - fixed");
    assert_eq!(retrieved.outcome, "success");
    assert_eq!(retrieved.created_at, 123);
    assert!(retrieved.updated_at >= 123);
    Ok(())
}

#[test]
fn test_incremental_delete_episode() -> TestResult {
    let store = test_store("test");

    let ep1 = Episode::new(
        "ep-1".to_string(),
        "debug api".to_string(),
        store.encoder().encode("debug api"),
        "Solution".to_string(),
        "success".to_string(),
    );
    let ep2 = Episode::new(
        "ep-2".to_string(),
        "fix memory".to_string(),
        store.encoder().encode("fix memory"),
        "Solution".to_string(),
        "success".to_string(),
    );
    store.store(ep1)?;
    store.store(ep2)?;

    assert_eq!(store.len(), 2);

    let deleted = store.delete_episode("ep-1");
    assert!(deleted, "Delete should return true");

    assert_eq!(store.len(), 1);
    assert!(store.get("ep-1").is_none(), "ep-1 should be deleted");
    assert!(store.get("ep-2").is_some(), "ep-2 should still exist");
    assert!(
        (store.q_table.get_q("ep-1") - 0.5).abs() < f32::EPSILON,
        "Deleted ep should have default Q"
    );
    assert!(
        (store.q_table.get_q("ep-2") - 0.5).abs() < f32::EPSILON,
        "Remaining ep should have Q"
    );
    Ok(())
}

#[test]
fn test_incremental_mark_accessed() -> TestResult {
    let store = test_store("test");

    let ep = Episode::new(
        "ep-1".to_string(),
        "debug api".to_string(),
        store.encoder().encode("debug api"),
        "Solution".to_string(),
        "success".to_string(),
    );
    store.store(ep)?;

    let retrieved = store
        .get("ep-1")
        .ok_or_else(|| std::io::Error::other("ep-1 should exist before mark_accessed"))?;
    assert_eq!(retrieved.retrieval_count, 0);
    assert_eq!(retrieved.success_count, 0);

    store.mark_accessed("ep-1");
    store.mark_accessed("ep-1");
    store.mark_accessed("ep-1");

    let retrieved = store
        .get("ep-1")
        .ok_or_else(|| std::io::Error::other("ep-1 should exist after mark_accessed"))?;
    assert_eq!(retrieved.retrieval_count, 3, "Should have 3 access counts");
    assert_eq!(
        retrieved.success_count, 0,
        "Access should not imply success"
    );
    Ok(())
}
