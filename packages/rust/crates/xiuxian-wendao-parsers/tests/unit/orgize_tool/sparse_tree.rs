use xiuxian_wendao_parsers::{
    OrgizeSparseTreeRenderOptions, OrgizeSparseTreeRequest, OrgizeSparseTreeVisibility,
    render_sparse_tree,
};

use super::support::tempdir_or_panic;

#[test]
fn render_sparse_tree_can_filter_done_tasks() {
    let temp = tempdir_or_panic();
    let path = temp.path().join("memory.org");
    std::fs::write(
        &path,
        concat!(
            "* TODO Active memory :agent:\n",
            "The active agent memory mentions sparse tree cards.\n",
            "* DONE Retired memory :agent:\n",
            "The retired agent memory should be filtered.\n",
        ),
    )
    .unwrap_or_else(|error| panic!("write memory: {error}"));

    let rendered = render_sparse_tree(&OrgizeSparseTreeRequest {
        paths: vec![path],
        text: Some("agent memory".to_string()),
        match_expression: Some("+agent".to_string()),
        visibility: OrgizeSparseTreeVisibility {
            exclude_done: true,
            exclude_archived: false,
        },
        include_comments: false,
        render: OrgizeSparseTreeRenderOptions {
            explain_skips: false,
        },
    })
    .unwrap_or_else(|error| panic!("render sparse tree: {error}"));

    assert!(
        rendered.contains("[SPARSE001] Match: Active memory"),
        "rendered: {rendered}"
    );
    assert!(!rendered.contains("Retired memory"), "rendered: {rendered}");
}
