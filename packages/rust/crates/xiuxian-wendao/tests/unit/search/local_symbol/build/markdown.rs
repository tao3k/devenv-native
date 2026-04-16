use std::collections::BTreeMap;
use std::thread;
use std::time::Duration;

use crate::search::local_symbol::build::plan_local_symbol_build;

use super::support::{demo_projects, only_partition, planning_service, singleton_replaced_path};

#[test]
fn local_symbol_build_ignores_metadata_only_markdown_edits_when_symbol_surface_is_unchanged() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs: {error}"));
    std::fs::write(
        project_root.join("docs/guide.md"),
        concat!(
            "# Alpha\n\n",
            "Body text.\n\n",
            "- [ ] Ship parser lane\n\n",
            "## Overview\n",
            ":PROPERTIES:\n",
            ":ID: alpha\n",
            ":OBSERVE: lang:rust \"fn $NAME()\"\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write guide: {error}"));
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
        .get("docs/guide.md")
        .unwrap_or_else(|| panic!("initial markdown local symbol fingerprint"));

    thread::sleep(Duration::from_millis(5));
    std::fs::write(
        project_root.join("docs/guide.md"),
        concat!(
            "# Alpha\n\n",
            "Body text with more prose.\n\n",
            "\n",
            "- [ ] Ship parser lane\n\n",
            "## Overview\n",
            ":PROPERTIES:\n",
            ":ID: alpha\n",
            ":OBSERVE: lang:rust \"fn $NAME()\"\n",
            ":END:\n\n",
        ),
    )
    .unwrap_or_else(|error| panic!("rewrite guide: {error}"));

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
        .get("docs/guide.md")
        .unwrap_or_else(|| panic!("updated markdown local symbol fingerprint"));

    assert_eq!(second.base_epoch, Some(7));
    assert!(second.partitions.is_empty());
    assert_ne!(first_fingerprint.size_bytes, second_fingerprint.size_bytes);
    assert_eq!(first_fingerprint.blake3, second_fingerprint.blake3);
}

#[test]
fn local_symbol_build_reparses_markdown_files_when_symbol_surface_changes() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs: {error}"));
    std::fs::write(
        project_root.join("docs/guide.md"),
        concat!(
            "# Alpha\n\n",
            "- [ ] Ship parser lane\n\n",
            "## Overview\n",
            ":PROPERTIES:\n",
            ":ID: alpha\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write guide: {error}"));
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

    thread::sleep(Duration::from_millis(5));
    std::fs::write(
        project_root.join("docs/guide.md"),
        concat!(
            "# Alpha\n\n",
            "- [ ] Ship parser lane\n\n",
            "## Implementation\n",
            ":PROPERTIES:\n",
            ":ID: beta\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("rewrite guide: {error}"));

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
        singleton_replaced_path("docs/guide.md")
    );
    assert!(
        changed_partition
            .changed_hits
            .iter()
            .any(|hit| hit.name == "Implementation"),
        "expected changed markdown heading hit"
    );
}
