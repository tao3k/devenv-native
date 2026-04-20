//! Integration tests for search result cache (`get_cached`, `set_cached`, hybrid).
//!
//! Tests share a static cache; run with:
//! `cargo test -p xiuxian-vector --test test_search_cache -- --test-threads=1`

use std::sync::{Mutex, OnceLock};

use xiuxian_vector::search_cache::{
    clear_cache, get_cached, get_cached_hybrid, set_cached, set_cached_hybrid,
};

fn with_cache_lock(test: impl FnOnce()) {
    static CACHE_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = match CACHE_TEST_LOCK.get_or_init(|| Mutex::new(())).lock() {
        Ok(guard) => guard,
        Err(error) => panic!("acquire search cache test lock: {error}"),
    };
    clear_cache();
    test();
}

#[test]
fn test_cache_miss_then_hit() {
    with_cache_lock(|| {
        let path = "/tmp/test";
        let table = "knowledge";
        let limit = 10;
        let vector = vec![0.1, 0.2, 0.3f32];

        assert!(get_cached(path, table, limit, None, &vector).is_none());

        let results = vec!["a".to_string(), "b".to_string()];
        set_cached(path, table, limit, None, &vector, results.clone());

        let cached = get_cached(path, table, limit, None, &vector);
        assert_eq!(cached, Some(results));
    });
}

#[test]
fn test_different_vectors_different_entries() {
    with_cache_lock(|| {
        let path = "/tmp/test";
        let table = "knowledge";
        let limit = 10;

        let v1 = vec![0.1f32, 0.2];
        let v2 = vec![0.2f32, 0.1];
        set_cached(path, table, limit, None, &v1, vec!["r1".into()]);
        set_cached(path, table, limit, None, &v2, vec!["r2".into()]);

        assert_eq!(
            get_cached(path, table, limit, None, &v1),
            Some(vec!["r1".to_string()])
        );
        assert_eq!(
            get_cached(path, table, limit, None, &v2),
            Some(vec!["r2".to_string()])
        );
    });
}

#[test]
fn test_hybrid_cache_miss_then_hit() {
    with_cache_lock(|| {
        let path = "/tmp/test";
        let table = "tools";
        let limit = 5;
        let vector = vec![0.5f32, 0.6];
        let query_text = "git commit";

        assert!(get_cached_hybrid(path, table, limit, &vector, query_text).is_none());

        let results = vec!["git.commit".into(), "git.status".into()];
        set_cached_hybrid(path, table, limit, &vector, query_text, results.clone());

        let cached = get_cached_hybrid(path, table, limit, &vector, query_text);
        assert_eq!(cached, Some(results));
    });
}
