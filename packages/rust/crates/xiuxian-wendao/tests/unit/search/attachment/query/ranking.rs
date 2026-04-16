use crate::search::attachment::query::lookup::{
    AttachmentCandidate, compare_candidates, retained_window,
};
use crate::search::ranking::trim_ranked_vec;

#[test]
fn trim_candidates_keeps_highest_ranked_attachment_hits() {
    let mut candidates = vec![
        AttachmentCandidate {
            id: "zeta".to_string(),
            score: 0.4,
            source_path: "docs/zeta.md".to_string(),
            attachment_path: "assets/zeta.png".to_string(),
        },
        AttachmentCandidate {
            id: "beta".to_string(),
            score: 0.9,
            source_path: "docs/beta.md".to_string(),
            attachment_path: "assets/beta.png".to_string(),
        },
        AttachmentCandidate {
            id: "alpha".to_string(),
            score: 0.9,
            source_path: "docs/alpha.md".to_string(),
            attachment_path: "assets/alpha.png".to_string(),
        },
    ];

    trim_ranked_vec(&mut candidates, 2, compare_candidates);

    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].attachment_path, "assets/alpha.png");
    assert_eq!(candidates[1].attachment_path, "assets/beta.png");
}

#[test]
fn retained_window_scales_with_limit() {
    assert_eq!(retained_window(0).target, 32);
    assert_eq!(retained_window(8).target, 32);
    assert_eq!(retained_window(32).target, 64);
}
