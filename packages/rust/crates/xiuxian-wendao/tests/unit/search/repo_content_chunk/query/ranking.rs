use std::collections::HashMap;

use crate::search::ranking::trim_ranked_string_map;
use crate::search::repo_content_chunk::query::{
    RepoContentChunkCandidate, candidate_path_key, compare_candidates, retained_window,
};

#[test]
fn trim_best_by_path_keeps_top_ranked_paths() {
    let mut best_by_path = HashMap::from([
        (
            "src/zeta.jl".to_string(),
            RepoContentChunkCandidate {
                path: "src/zeta.jl".to_string(),
                language: Some("julia".to_string()),
                line_number: 30,
                line_text: "zeta".to_string(),
                score: 0.72,
                exact_match: false,
            },
        ),
        (
            "src/beta.jl".to_string(),
            RepoContentChunkCandidate {
                path: "src/beta.jl".to_string(),
                language: Some("julia".to_string()),
                line_number: 20,
                line_text: "beta".to_string(),
                score: 0.73,
                exact_match: true,
            },
        ),
        (
            "src/alpha.jl".to_string(),
            RepoContentChunkCandidate {
                path: "src/alpha.jl".to_string(),
                language: Some("julia".to_string()),
                line_number: 10,
                line_text: "alpha".to_string(),
                score: 0.73,
                exact_match: true,
            },
        ),
    ]);

    trim_ranked_string_map(&mut best_by_path, 2, compare_candidates, candidate_path_key);

    let mut retained = best_by_path.into_values().collect::<Vec<_>>();
    retained.sort_by(compare_candidates);
    assert_eq!(retained.len(), 2);
    assert_eq!(retained[0].path, "src/alpha.jl");
    assert_eq!(retained[1].path, "src/beta.jl");
}

#[test]
fn retained_window_scales_with_limit() {
    assert_eq!(retained_window(0).target, 128);
    assert_eq!(retained_window(4).target, 128);
    assert_eq!(retained_window(64).target, 512);
}
