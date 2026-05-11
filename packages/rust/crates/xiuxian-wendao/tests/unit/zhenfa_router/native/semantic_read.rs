use std::collections::HashMap;
use std::sync::Arc;

use crate::link_graph::{MarkdownBlock, PageIndexMeta, PageIndexNode};

use super::{MatchType, find_node_by_id, match_type_to_string};

fn make_node(node_id: &str, title: &str, children: Vec<PageIndexNode>) -> PageIndexNode {
    PageIndexNode {
        node_id: node_id.to_string().into(),
        parent_id: None,
        title: title.to_string(),
        level: 1,
        text: Arc::from(title),
        summary: None,
        children,
        metadata: PageIndexMeta {
            line_range: (1, 1),
            byte_range: Some((0, 0)),
            structural_path: vec![title.to_string()],
            content_hash: None,
            attributes: HashMap::new(),
            token_count: 0,
            is_thinned: false,
            logbook: vec![],
            observations: vec![],
        },
        blocks: Vec::<MarkdownBlock>::new(),
    }
}

#[test]
fn match_type_to_string_covers_variants() {
    assert_eq!(match_type_to_string(MatchType::Exact), "exact");
    assert_eq!(match_type_to_string(MatchType::Suffix), "suffix");
    assert_eq!(
        match_type_to_string(MatchType::TitleSubstring),
        "title_substring"
    );
    assert_eq!(match_type_to_string(MatchType::TitleFuzzy), "title_fuzzy");
    assert_eq!(
        match_type_to_string(MatchType::HashFallback),
        "hash_fallback"
    );
    assert_eq!(
        match_type_to_string(MatchType::CaseInsensitive),
        "case_insensitive"
    );
}

#[test]
fn find_node_by_id_recurses_into_children() {
    let child = make_node("child", "Child", Vec::new());
    let root = make_node("root", "Root", vec![child]);

    let Some(found) = find_node_by_id(&[root], "child") else {
        panic!("child node should be found");
    };
    assert_eq!(found.node_id, "child");
    assert_eq!(found.title, "Child");
}
