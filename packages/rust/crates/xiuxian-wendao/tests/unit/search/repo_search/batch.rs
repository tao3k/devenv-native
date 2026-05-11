use crate::search::contracts::{SearchHit, StudioNavigationTarget};

#[test]
fn repo_search_batch_from_hits_rejects_navigation_lines_outside_i32_range() {
    let overflow_line = usize::try_from(i32::MAX)
        .unwrap_or_else(|error| panic!("i32::MAX should fit usize: {error}"))
        + 1;
    let hits = vec![SearchHit {
        stem: "lib.rs".to_string(),
        title: None,
        path: "src/lib.rs".to_string().into(),
        doc_type: None,
        tags: Vec::new(),
        score: 1.0,
        best_section: None,
        match_reason: None,
        hierarchical_uri: None,
        hierarchy: None,
        saliency_score: None,
        audit_status: None,
        verification_state: None,
        implicit_backlinks: None,
        implicit_backlink_items: None,
        navigation_target: Some(StudioNavigationTarget {
            path: "src/lib.rs".to_string().into(),
            category: "symbol".to_string(),
            project_name: None,
            root_label: None,
            line: Some(overflow_line),
            line_end: None,
            column: None,
        }),
    }];

    let Err(error) = super::repo_search_batch_from_hits(&hits) else {
        panic!("out-of-range navigation lines should fail");
    };

    assert_eq!(
        error,
        format!("repo-search hit `src/lib.rs` line `{overflow_line}` exceeds i32 range")
    );
}
