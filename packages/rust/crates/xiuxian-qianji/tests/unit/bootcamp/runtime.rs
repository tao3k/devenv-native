use super::build_link_graph_index_with_builders;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use xiuxian_wendao::link_graph::LinkGraphIndex;

#[test]
fn build_link_graph_index_falls_back_to_plain_build_when_cache_bootstrap_fails() {
    let root = tempdir().unwrap_or_else(|error| panic!("tempdir should succeed: {error}"));
    let index = build_link_graph_index_with_builders(
        root.path(),
        root.path(),
        |_| Err("cache unavailable".to_string()),
        LinkGraphIndex::build,
    )
    .unwrap_or_else(|error| panic!("plain build fallback should succeed: {error}"));
    assert_eq!(
        fs::canonicalize(index.root())
            .unwrap_or_else(|error| panic!("index root should canonicalize: {error}")),
        fs::canonicalize(root.path())
            .unwrap_or_else(|error| panic!("temp root should canonicalize: {error}"))
    );
}

#[test]
fn build_link_graph_index_tries_fallback_root_after_primary_root_failure() {
    let fallback_root = tempdir().unwrap_or_else(|error| panic!("tempdir should succeed: {error}"));
    let fallback_root_path = fallback_root.path().to_path_buf();
    let primary_root = fallback_root.path().join("missing-primary-root");
    let seen_roots = Arc::new(Mutex::new(Vec::<PathBuf>::new()));

    let index = build_link_graph_index_with_builders(
        primary_root.as_path(),
        fallback_root_path.as_path(),
        |_| Err("cache unavailable".to_string()),
        {
            let seen_roots = Arc::clone(&seen_roots);
            let fallback_root_path = fallback_root_path.clone();
            move |root: &Path| {
                seen_roots
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(root.to_path_buf());
                if root == fallback_root_path.as_path() {
                    LinkGraphIndex::build(root)
                } else {
                    Err("forced primary failure".to_string())
                }
            }
        },
    )
    .unwrap_or_else(|error| panic!("fallback root build should succeed: {error}"));

    let roots = seen_roots
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(
        roots,
        vec![primary_root.clone(), fallback_root_path.clone()]
    );
    assert_eq!(
        fs::canonicalize(index.root())
            .unwrap_or_else(|error| panic!("index root should canonicalize: {error}")),
        fs::canonicalize(fallback_root_path.as_path())
            .unwrap_or_else(|error| panic!("fallback root should canonicalize: {error}"))
    );
}
