use std::collections::BTreeMap;
use std::thread;
use std::time::Duration;

use crate::search::local_symbol::build::plan_local_symbol_build;

use super::support::{
    count_changed_hits, demo_projects, only_partition, planning_service, singleton_replaced_path,
    write_demo_source,
};

#[test]
fn plan_local_symbol_build_only_reparses_changed_files() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    write_demo_source(project_root, "src/lib.rs", "fn alpha() {}\n");
    write_demo_source(project_root, "src/extra.rs", "fn gamma() {}\n");
    let projects = demo_projects();
    let service = planning_service(project_root);

    let first = plan_local_symbol_build(
        &service,
        project_root,
        project_root,
        &projects,
        None,
        &BTreeMap::new(),
    );
    assert_eq!(first.base_epoch, None);
    assert_eq!(count_changed_hits(&first), 2);

    thread::sleep(Duration::from_millis(5));
    write_demo_source(project_root, "src/lib.rs", "fn beta() {}\n");

    let second = plan_local_symbol_build(
        &service,
        project_root,
        project_root,
        &projects,
        Some(7),
        &first.file_fingerprints,
    );
    assert_eq!(second.base_epoch, Some(7));
    let changed_partition = only_partition(&second);
    assert_eq!(
        changed_partition.replaced_paths,
        singleton_replaced_path("src/lib.rs")
    );
    assert_eq!(changed_partition.changed_hits.len(), 1);
    assert_eq!(changed_partition.changed_hits[0].name, "beta");
}

#[test]
fn plan_local_symbol_build_ignores_metadata_only_edits_when_ast_hits_are_unchanged() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    write_demo_source(project_root, "src/lib.rs", "fn alpha() {}\n");
    let projects = demo_projects();
    let service = planning_service(project_root);

    let first = plan_local_symbol_build(
        &service,
        project_root,
        project_root,
        &projects,
        None,
        &BTreeMap::new(),
    );
    let first_fingerprint = first
        .file_fingerprints
        .get("src/lib.rs")
        .unwrap_or_else(|| panic!("initial local symbol fingerprint"));

    thread::sleep(Duration::from_millis(5));
    write_demo_source(project_root, "src/lib.rs", "fn alpha() {}\n\n");

    let second = plan_local_symbol_build(
        &service,
        project_root,
        project_root,
        &projects,
        Some(7),
        &first.file_fingerprints,
    );
    let second_fingerprint = second
        .file_fingerprints
        .get("src/lib.rs")
        .unwrap_or_else(|| panic!("updated local symbol fingerprint"));

    assert_eq!(second.base_epoch, Some(7));
    assert!(second.partitions.is_empty());
    assert_ne!(first_fingerprint.size_bytes, second_fingerprint.size_bytes);
    assert_eq!(first_fingerprint.blake3, second_fingerprint.blake3);
}
