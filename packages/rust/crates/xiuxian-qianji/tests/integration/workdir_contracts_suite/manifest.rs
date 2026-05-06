use tempfile::TempDir;
use xiuxian_qianji::{load_workdir_manifest, parse_workdir_manifest};

use super::{create_valid_workdir, valid_workdir_manifest};

#[test]
fn bounded_workdir_manifest_parses_compact_contract() {
    let manifest = parse_workdir_manifest(valid_workdir_manifest())
        .unwrap_or_else(|error| panic!("compact work-surface manifest should parse: {error}"));

    assert_eq!(manifest.version, 1);
    assert_eq!(manifest.plan.name, "demo-plan");
    assert_eq!(
        manifest.plan.surface,
        vec![
            "flowchart.mmd".to_string(),
            "blueprint".to_string(),
            "plan".to_string()
        ]
    );
    assert_eq!(
        manifest.check.flowchart,
        vec!["blueprint".to_string(), "plan".to_string()]
    );
}
#[test]
fn bounded_workdir_manifest_rejects_missing_flowchart_surface() {
    let error = parse_workdir_manifest(
        r#"
version = 1

[plan]
name = "broken"
surface = ["blueprint", "plan"]

[check]
require = ["flowchart.mmd", "blueprint", "plan"]
flowchart = ["blueprint", "plan"]
"#,
    )
    .err()
    .unwrap_or_else(|| panic!("missing flowchart surface should fail"));

    assert!(
        error
            .to_string()
            .contains("`plan.surface` must include `flowchart.mmd`")
    );
}
#[test]
fn load_workdir_manifest_reads_real_file() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    let manifest = load_workdir_manifest(workdir.join("qianji.toml"))
        .unwrap_or_else(|error| panic!("root manifest file should load: {error}"));

    assert_eq!(manifest.plan.name, "demo-plan");
    assert_eq!(manifest.check.require.len(), 5);
}
