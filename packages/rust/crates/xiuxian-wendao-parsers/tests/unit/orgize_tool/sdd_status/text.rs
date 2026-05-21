use xiuxian_wendao_parsers::{OrgizeSddStatusRequest, render_sdd_status};

use crate::orgize_tool::support::tempdir_or_panic;

#[test]
fn render_sdd_status_uses_org_native_parent_edges() {
    let temp = tempdir_or_panic();
    let path = temp.path().join("sdd.org");
    std::fs::write(
        &path,
        concat!(
            "* System SDD :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: review\n",
            ":SDD_CONCERN: Agent planning architecture boundaries.\n",
            ":END:\n",
            "** Runtime View :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-55a2-70c0-98db-7ac2c4d80d78\n",
            ":SDD_KIND: view\n",
            ":SDD_PARENT: [[id:018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11][System SDD]]\n",
            ":SDD_VIEWPOINT: runtime\n",
            ":SDD_CONCERN: Recovery query and design-governance flow.\n",
            ":SDD_STATUS: review\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write sdd org: {error}"));

    let rendered = render_sdd_status(&OrgizeSddStatusRequest {
        paths: vec![path],
        issues_only: false,
    })
    .unwrap_or_else(|error| panic!("render sdd status: {error}"));

    assert!(rendered.contains("[SDD]"), "rendered: {rendered}");
    assert!(
        rendered.contains("architecture nodes: 2"),
        "rendered: {rendered}"
    );
    assert!(
        rendered.contains("summary: kinds={system=1, view=1}; statuses={review=2}; issues=0"),
        "rendered: {rendered}"
    );
    assert!(rendered.contains("tree:"), "rendered: {rendered}");
    assert!(
        rendered.contains("- view review: Runtime View"),
        "rendered: {rendered}"
    );
    assert!(
        rendered.contains("parent: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11 (System SDD)"),
        "rendered: {rendered}"
    );
    assert!(
        rendered.contains("viewpoint: runtime"),
        "rendered: {rendered}"
    );
    assert!(
        rendered.contains("diagnostics:\n- no issues"),
        "rendered: {rendered}"
    );
}

#[test]
fn render_sdd_status_reports_missing_path_recovery() {
    let temp = tempdir_or_panic();
    let path = temp.path().join("agent").join("sdd");

    let rendered = render_sdd_status(&OrgizeSddStatusRequest {
        paths: vec![path],
        issues_only: false,
    })
    .unwrap_or_else(|error| panic!("render missing sdd status: {error}"));

    assert!(
        rendered.contains("architecture nodes: 0"),
        "rendered: {rendered}"
    );
    assert!(rendered.contains("missing-path"), "rendered: {rendered}");
    assert!(
        rendered.contains("copy `.agent/sdd/_architecture_template.org`"),
        "rendered: {rendered}"
    );
}

#[test]
fn render_sdd_status_surfaces_template_and_parent_diagnostics() {
    let temp = tempdir_or_panic();
    let path = temp.path().join("sdd.org");
    std::fs::write(
        &path,
        concat!(
            "* System SDD :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 00000000-0000-7000-8000-000000000001\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: draft\n",
            ":END:\n",
            "** Runtime View :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-55a2-70c0-98db-7ac2c4d80d78\n",
            ":SDD_KIND: view\n",
            ":SDD_PARENT: [[id:018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11][Missing System]]\n",
            ":SDD_STATUS: review\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write diagnostic sdd org: {error}"));

    let rendered = render_sdd_status(&OrgizeSddStatusRequest {
        paths: vec![path],
        issues_only: false,
    })
    .unwrap_or_else(|error| panic!("render diagnostic sdd status: {error}"));

    assert!(
        rendered
            .contains("summary: kinds={system=1, view=1}; statuses={draft=1, review=1}; issues="),
        "rendered: {rendered}"
    );
    assert!(rendered.contains("[template-id]"), "rendered: {rendered}");
    assert!(rendered.contains("[orphan-parent]"), "rendered: {rendered}");
    assert!(
        rendered.contains("[missing-viewpoint]"),
        "rendered: {rendered}"
    );
    assert!(
        rendered.contains("[missing-concern]"),
        "rendered: {rendered}"
    );
}

#[test]
fn render_sdd_status_issues_only_filters_clean_files() {
    let temp = tempdir_or_panic();
    let clean = temp.path().join("clean.org");
    let drifted = temp.path().join("drifted.org");
    std::fs::write(
        &clean,
        concat!(
            "* Clean System :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: review\n",
            ":SDD_CONCERN: Clean status should be filtered.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write clean sdd org: {error}"));
    std::fs::write(
        &drifted,
        concat!(
            "* Drifted View :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-55a2-70c0-98db-7ac2c4d80d78\n",
            ":SDD_KIND: view\n",
            ":SDD_STATUS: review\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write drifted sdd org: {error}"));

    let rendered = render_sdd_status(&OrgizeSddStatusRequest {
        paths: vec![temp.path().to_path_buf()],
        issues_only: true,
    })
    .unwrap_or_else(|error| panic!("render issues-only sdd status: {error}"));

    assert!(!rendered.contains("Clean System"), "rendered: {rendered}");
    assert!(rendered.contains("Drifted View"), "rendered: {rendered}");
    assert!(!rendered.contains("tree:"), "rendered: {rendered}");
    assert!(
        rendered.contains("[missing-parent]"),
        "rendered: {rendered}"
    );
    assert!(
        rendered.contains("[missing-viewpoint]"),
        "rendered: {rendered}"
    );
}

#[test]
fn render_sdd_status_issues_only_reports_no_issues() {
    let temp = tempdir_or_panic();
    let path = temp.path().join("clean.org");
    std::fs::write(
        &path,
        concat!(
            "* Clean System :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: review\n",
            ":SDD_CONCERN: Clean status should report no issues.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write clean sdd org: {error}"));

    let rendered = render_sdd_status(&OrgizeSddStatusRequest {
        paths: vec![path],
        issues_only: true,
    })
    .unwrap_or_else(|error| panic!("render no-issues sdd status: {error}"));

    assert_eq!(rendered, "[ok] orgize sdd status: no issues\n");
}
