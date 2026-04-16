use crate::gateway::studio::search::project_metadata_for_path;
use crate::gateway::studio::types::UiProjectConfig;

#[test]
fn project_metadata_prefers_more_specific_scope_root_label() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    let packages_root = project_root.join("packages");
    std::fs::create_dir_all(packages_root.join("rust/crates/demo/src"))
        .unwrap_or_else(|error| panic!("packages tree: {error}"));

    let metadata = project_metadata_for_path(
        project_root,
        project_root,
        &[UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec![".".to_string(), "packages".to_string()],
        }],
        "packages/rust/crates/demo/src/lib.rs",
    );

    assert_eq!(metadata.project_name.as_deref(), Some("kernel"));
    assert_eq!(metadata.root_label.as_deref(), Some("packages"));
}
