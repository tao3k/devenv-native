use std::fs;

use tempfile::TempDir;
use xiuxian_qianji::{
    WorkdirMarkdownSurface, WorkdirVisibleSurfaceKind, build_workdir_check_follow_up_query,
    check_workdir, render_workdir_check_markdown, render_workdir_show, show_workdir,
};

use super::{
    assert_common_diagnostic_shape, assert_common_show_shape, create_step_aware_workdir,
    create_valid_workdir, write_file,
};

#[test]
fn show_workdir_reports_top_level_surface_state() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    fs::remove_dir_all(workdir.join("plan"))
        .unwrap_or_else(|error| panic!("should remove plan dir for show test: {error}"));

    let show = show_workdir(&workdir)
        .unwrap_or_else(|error| panic!("show surface should still render: {error}"));

    assert_eq!(show.plan_name, "demo-plan");
    assert_eq!(show.surfaces[0].surface, "flowchart.mmd");
    assert_eq!(show.surfaces[0].kind, WorkdirVisibleSurfaceKind::File);
    assert_eq!(show.surfaces[1].kind, WorkdirVisibleSurfaceKind::Directory);
    assert_eq!(show.surfaces[2].kind, WorkdirVisibleSurfaceKind::Missing);

    let rendered = render_workdir_show(&show);
    assert_common_show_shape(&rendered);
    assert!(rendered.contains("# Work Surface"));
    assert!(rendered.contains("## flowchart.mmd"));
    assert!(rendered.contains("Status: missing"));
}
#[test]
fn check_workdir_accepts_valid_surface() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("valid work surface should check: {error}"));

    assert!(report.is_valid());
    let rendered = render_workdir_check_markdown(&report);
    assert!(rendered.contains("# Validation Passed"));
}
#[test]
fn check_workdir_accepts_step_aware_current_node_without_future_outputs() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_step_aware_workdir(&temp_dir);

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("step-aware work surface should check: {error}"));

    assert!(report.is_valid());
    let rendered = render_workdir_check_markdown(&report);
    assert!(rendered.contains("# Validation Passed"));
}
#[test]
fn check_workdir_blocks_allowed_next_drift_for_step_aware_surface() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_step_aware_workdir(&temp_dir);
    write_file(
        &workdir.join("state/allowed_next.json"),
        "[\"done_gate\"]\n",
    );

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("step-aware drift should still report: {error}"));

    assert!(!report.is_valid());
    let rendered = render_workdir_check_markdown(&report);
    assert_common_diagnostic_shape(&rendered);
    assert!(rendered.contains("Allowed-next drift"));
    assert!(rendered.contains("current node `methods_extract`"));
    assert!(rendered.contains("`results_extract`"));
    assert!(rendered.contains("`done gate`"));
}
#[test]
fn check_workdir_reports_missing_glob_matches_and_backbone_conflicts() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    fs::remove_file(workdir.join("plan/tasks.md"))
        .unwrap_or_else(|error| panic!("should remove plan markdown: {error}"));
    write_file(
        &workdir.join("flowchart.mmd"),
        "flowchart LR\n  plan --> blueprint\n",
    );

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("invalid work surface should still report: {error}"));

    assert!(!report.is_valid());
    let rendered = render_workdir_check_markdown(&report);
    assert_common_diagnostic_shape(&rendered);
    assert!(rendered.contains("Missing required glob matches"));
    assert!(rendered.contains("Missing flowchart backbone"));
    assert!(rendered.contains("blueprint --> plan"));
    assert!(rendered.contains("## Follow-up Query"));
    assert!(rendered.contains("Surfaces: blueprint, plan"));
    assert!(rendered.contains(
        "select path, surface, surface_kind, heading_path, skeleton \
from markdown \
where surface in ('blueprint', 'plan') \
order by surface, path, heading_path"
    ));
}
#[test]
fn check_workdir_render_includes_follow_up_query_on_failure() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    fs::remove_file(workdir.join("plan/tasks.md"))
        .unwrap_or_else(|error| panic!("should remove plan markdown: {error}"));

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("invalid work surface should still report: {error}"));
    let rendered = render_workdir_check_markdown(&report);

    assert!(rendered.contains("## Follow-up Query"));
    assert!(rendered.contains("Surfaces: plan"));
    assert!(rendered.contains(
        "select path, surface, surface_kind, heading_path, skeleton \
from markdown \
where surface = 'plan' \
order by surface, path, heading_path"
    ));
}
#[test]
fn workdir_check_follow_up_query_stays_surface_bounded() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    fs::remove_file(workdir.join("plan/tasks.md"))
        .unwrap_or_else(|error| panic!("should remove plan markdown: {error}"));

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("invalid work surface should still report: {error}"));
    let follow_up = build_workdir_check_follow_up_query(&report)
        .unwrap_or_else(|| panic!("failing report should derive follow-up query"));

    assert_eq!(follow_up.workdir, workdir);
    assert_eq!(follow_up.surfaces, vec![WorkdirMarkdownSurface::Plan]);
    assert_eq!(
        follow_up.query_text,
        "select path, surface, surface_kind, heading_path, skeleton \
from markdown \
where surface = 'plan' \
order by surface, path, heading_path"
    );
}
