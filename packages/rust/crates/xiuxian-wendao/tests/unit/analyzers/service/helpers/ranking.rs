use crate::analyzers::ImportKind;
use crate::analyzers::{
    example_match_score, import_match_score, module_match_score, normalized_rank_score,
    symbol_match_score,
};

use super::fixtures::import_record;

#[test]
fn ranking_helpers_distinguish_common_match_shapes() {
    assert!((normalized_rank_score(0, 3) - 1.0).abs() < f64::EPSILON);
    assert!((normalized_rank_score(3, 3) - 0.25).abs() < f64::EPSILON);
    assert_eq!(
        module_match_score("alpha", "alpha.beta", "src/alpha/beta.rs"),
        Some(1)
    );
    assert_eq!(
        symbol_match_score(
            "solve",
            "solve",
            "alpha.beta::solve",
            "src/alpha/beta.rs",
            "fn solve()"
        ),
        Some(0)
    );
    assert_eq!(
        example_match_score(
            "solve",
            "solve example",
            "examples/solve.rs",
            "solve summary",
            &[String::from("related-symbol")],
            &[String::from("related-module")],
        ),
        Some(1)
    );
    let import = import_record("repo-a", "mod-a", "solver", "sciml-solver", "alpha.beta");
    assert_eq!(import.kind, ImportKind::Module);
    assert_eq!(
        import_match_score(Some("sciml-solver"), Some("alpha.beta"), &import),
        Some(0)
    );
}
