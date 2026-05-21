use xiuxian_wendao_parsers::{
    OrgizeSddGraphDiffRequest, count_sdd_graph_drift, render_sdd_graph_diff,
};

use super::support::tempdir_or_panic;

#[test]
fn render_sdd_graph_diff_reports_aligned_outline_edges() {
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
            ":SDD_CONCERN: Graph diff alignment.\n",
            ":END:\n",
            "** Runtime View :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-55a2-70c0-98db-7ac2c4d80d78\n",
            ":SDD_KIND: view\n",
            ":SDD_PARENT: [[id:018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11][System SDD]]\n",
            ":SDD_VIEWPOINT: runtime\n",
            ":SDD_CONCERN: Parent edge matches outline nesting.\n",
            ":SDD_STATUS: review\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write aligned graph sdd org: {error}"));

    let rendered = render_sdd_graph_diff(&OrgizeSddGraphDiffRequest { paths: vec![path] })
        .unwrap_or_else(|error| panic!("render sdd graph diff: {error}"));

    assert!(rendered.contains("[SDD-GRAPH]"), "rendered: {rendered}");
    assert!(
        rendered.contains("summary: {aligned=1, root=1}; drift=0"),
        "rendered: {rendered}"
    );
    assert!(
        rendered.contains("- aligned: Runtime View"),
        "rendered: {rendered}"
    );
    assert!(
        rendered.contains("semantic: System SDD"),
        "rendered: {rendered}"
    );
    assert!(
        rendered.contains("outline: System SDD"),
        "rendered: {rendered}"
    );
}

#[test]
fn render_sdd_graph_diff_reports_semantic_moves() {
    let temp = tempdir_or_panic();
    let path = temp.path().join("sdd.org");
    std::fs::write(
        &path,
        concat!(
            "* System A :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: review\n",
            ":SDD_CONCERN: Physical outline parent.\n",
            ":END:\n",
            "** Runtime View :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-55a2-70c0-98db-7ac2c4d80d78\n",
            ":SDD_KIND: view\n",
            ":SDD_PARENT: [[id:018f3f9c-4c78-7f24-bc2c-e1aa0d7cb881][System B]]\n",
            ":SDD_VIEWPOINT: runtime\n",
            ":SDD_CONCERN: Semantic parent differs from outline parent.\n",
            ":SDD_STATUS: review\n",
            ":END:\n",
            "* System B :sdd:\n",
            ":PROPERTIES:\n",
            ":ID: 018f3f9c-4c78-7f24-bc2c-e1aa0d7cb881\n",
            ":SDD_KIND: system\n",
            ":SDD_STATUS: review\n",
            ":SDD_CONCERN: Semantic parent.\n",
            ":END:\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write moved graph sdd org: {error}"));

    let rendered = render_sdd_graph_diff(&OrgizeSddGraphDiffRequest {
        paths: vec![path.clone()],
    })
    .unwrap_or_else(|error| panic!("render moved sdd graph diff: {error}"));
    let drift = count_sdd_graph_drift(&OrgizeSddGraphDiffRequest { paths: vec![path] })
        .unwrap_or_else(|error| panic!("count graph drift: {error}"));

    assert!(
        rendered.contains("summary: {root=2, semantic-move=1}; drift=1"),
        "rendered: {rendered}"
    );
    assert!(
        rendered.contains("- semantic-move: Runtime View"),
        "rendered: {rendered}"
    );
    assert!(
        rendered.contains("semantic: System B"),
        "rendered: {rendered}"
    );
    assert!(
        rendered.contains("outline: System A"),
        "rendered: {rendered}"
    );
    assert_eq!(drift, 1);
}
