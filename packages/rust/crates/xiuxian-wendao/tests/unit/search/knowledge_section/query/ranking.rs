use std::collections::HashMap;

use crate::search::knowledge_section::query::lookup::{
    KnowledgeCandidate, candidate_path_key, compare_candidates, retained_window,
};
use crate::search::ranking::trim_ranked_string_map;

#[test]
fn trim_best_by_path_keeps_highest_ranked_hits() {
    let mut best_by_path = HashMap::from([
        (
            "notes/zeta.md".to_string(),
            KnowledgeCandidate {
                id: "zeta".to_string(),
                path: "notes/zeta.md".to_string().into(),
                stem: "zeta".to_string(),
                score: 0.82,
            },
        ),
        (
            "notes/beta.md".to_string(),
            KnowledgeCandidate {
                id: "beta".to_string(),
                path: "notes/beta.md".to_string().into(),
                stem: "beta".to_string(),
                score: 0.95,
            },
        ),
        (
            "notes/alpha.md".to_string(),
            KnowledgeCandidate {
                id: "alpha".to_string(),
                path: "notes/alpha.md".to_string().into(),
                stem: "alpha".to_string(),
                score: 0.95,
            },
        ),
    ]);

    trim_ranked_string_map(&mut best_by_path, 2, compare_candidates, candidate_path_key);

    let mut retained = best_by_path.into_values().collect::<Vec<_>>();
    retained.sort_by(compare_candidates);
    assert_eq!(retained.len(), 2);
    assert_eq!(retained[0].path, "notes/alpha.md");
    assert_eq!(retained[1].path, "notes/beta.md");
}

#[test]
fn retained_window_scales_with_limit() {
    assert_eq!(retained_window(0).target, 128);
    assert_eq!(retained_window(4).target, 128);
    assert_eq!(retained_window(64).target, 512);
}
