use super::{fingerprint_note_projects, fingerprint_source_projects};
use crate::search::contracts::UiProjectConfig;

#[test]
fn fingerprint_source_projects_ignores_skipped_directories() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    std::fs::create_dir_all(project_root.join("node_modules/pkg"))
        .unwrap_or_else(|error| panic!("create skipped dir: {error}"));
    std::fs::write(project_root.join("src/lib.rs"), "fn alpha() {}\n")
        .unwrap_or_else(|error| panic!("write source file: {error}"));
    std::fs::write(
        project_root.join("node_modules/pkg/index.js"),
        "ignored();\n",
    )
    .unwrap_or_else(|error| panic!("write skipped source file: {error}"));
    let projects = vec![UiProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];

    let first = fingerprint_source_projects(project_root, project_root, &projects);
    std::fs::write(
        project_root.join("node_modules/pkg/index.js"),
        "ignored_again();\n",
    )
    .unwrap_or_else(|error| panic!("rewrite skipped source file: {error}"));
    let second = fingerprint_source_projects(project_root, project_root, &projects);
    assert_eq!(first, second);
}

#[test]
fn fingerprint_note_projects_changes_when_note_metadata_changes() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("notes"))
        .unwrap_or_else(|error| panic!("create notes dir: {error}"));
    std::fs::write(project_root.join("notes/test.md"), "# title\n")
        .unwrap_or_else(|error| panic!("write note file: {error}"));
    let projects = vec![UiProjectConfig {
        name: "demo".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];

    let first = fingerprint_note_projects(project_root, project_root, &projects);
    std::fs::write(project_root.join("notes/test.md"), "# title\nbody\n")
        .unwrap_or_else(|error| panic!("rewrite note file: {error}"));
    let second = fingerprint_note_projects(project_root, project_root, &projects);
    assert_ne!(first, second);
}
