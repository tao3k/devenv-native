use crate::search::local_symbol::build::fingerprint_projects;

use super::support::{demo_projects, write_demo_source};

#[test]
fn fingerprint_projects_changes_when_scanned_file_metadata_changes() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    write_demo_source(project_root, "src/lib.rs", "fn alpha() {}\n");
    write_demo_source(project_root, "node_modules/pkg/index.js", "ignored();\n");

    let projects = demo_projects();
    let first = fingerprint_projects(project_root, project_root, &projects);
    write_demo_source(
        project_root,
        "node_modules/pkg/index.js",
        "ignored-again();\n",
    );
    let after_skipped_change = fingerprint_projects(project_root, project_root, &projects);
    assert_eq!(first, after_skipped_change);

    write_demo_source(project_root, "src/lib.rs", "fn alpha() {}\nfn beta() {}\n");
    let second = fingerprint_projects(project_root, project_root, &projects);
    assert_ne!(first, second);
}
