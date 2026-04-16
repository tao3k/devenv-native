use crate::search::ranking::trim_ranked_vec;
use crate::search::repo_entity::query::lookup::{
    RepoEntityCandidate, compare_candidates, retained_window,
};

#[test]
fn trim_candidates_keeps_highest_ranked_entries() {
    let mut candidates = vec![
        RepoEntityCandidate {
            id: "example:1".to_string(),
            score: 0.50,
            entity_kind: "example".to_string(),
            name: "zeta".to_string(),
            path: "src/zeta.rs".to_string(),
        },
        RepoEntityCandidate {
            id: "symbol:1".to_string(),
            score: 0.93,
            entity_kind: "symbol".to_string(),
            name: "beta".to_string(),
            path: "src/beta.rs".to_string(),
        },
        RepoEntityCandidate {
            id: "module:1".to_string(),
            score: 0.93,
            entity_kind: "module".to_string(),
            name: "alpha".to_string(),
            path: "src/alpha.rs".to_string(),
        },
    ];

    trim_ranked_vec(&mut candidates, 2, compare_candidates);

    assert_eq!(candidates.len(), 2);
    assert!(
        candidates
            .windows(2)
            .all(|pair| compare_candidates(&pair[0], &pair[1]).is_le())
    );
    assert_eq!(candidates[0].entity_kind, "symbol");
    assert_eq!(candidates[1].entity_kind, "module");
}

#[test]
fn retained_window_scales_with_limit() {
    assert_eq!(retained_window(0).target, 256);
    assert_eq!(retained_window(4).target, 256);
    assert_eq!(retained_window(64).target, 512);
}
