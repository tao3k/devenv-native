use crate::search::real_repo_precision::{
    RealRepoGoldQueryKind, RealRepoPrecisionSyncMode, evaluate_gold_query_paths,
};

use super::support::gold_query;

#[test]
fn sync_mode_parser_defaults_to_status() {
    assert_eq!(
        RealRepoPrecisionSyncMode::parse(None),
        RealRepoPrecisionSyncMode::Status
    );
    assert_eq!(
        RealRepoPrecisionSyncMode::parse(Some("unknown")),
        RealRepoPrecisionSyncMode::Status
    );
    assert_eq!(
        RealRepoPrecisionSyncMode::parse(Some("ensure")),
        RealRepoPrecisionSyncMode::Ensure
    );
    assert_eq!(
        RealRepoPrecisionSyncMode::parse(Some("REFRESH")),
        RealRepoPrecisionSyncMode::Refresh
    );
    assert_eq!(RealRepoGoldQueryKind::parse_filter(None), None);
    assert_eq!(
        RealRepoGoldQueryKind::parse_filter(Some("link_graph")),
        Some(RealRepoGoldQueryKind::LinkGraph)
    );
    assert_eq!(RealRepoGoldQueryKind::parse_filter(Some("all")), None);
}

#[test]
fn evaluator_passes_when_expected_paths_are_returned() {
    let query = gold_query(Some("docs/rfcs/rfc.md"));
    let hits = vec![
        "docs/rfcs/rfc.md".to_string(),
        "packages/rust/crates/xiuxian-wendao/README.md".to_string(),
    ];

    let receipt = evaluate_gold_query_paths(&query, hits);

    assert!(receipt.passed);
    assert_eq!(receipt.query_kind, "link_graph");
    assert!(receipt.missing_paths.is_empty());
    assert_eq!(
        receipt.observed_top_path.as_deref(),
        Some("docs/rfcs/rfc.md")
    );
    assert_eq!(receipt.best_required_path_rank, Some(0));
    assert_eq!(receipt.required_path_recall_at_1_bps, 10_000);
    assert_eq!(receipt.required_path_recall_at_3_bps, 10_000);
    assert_eq!(receipt.required_path_recall_at_5_bps, 10_000);
    assert_eq!(receipt.required_path_recall_at_10_bps, 10_000);
    assert_eq!(receipt.mean_required_path_reciprocal_rank_bps, 10_000);
}

#[test]
fn evaluator_reports_missing_paths_and_top_path_mismatch() {
    let query = gold_query(Some("docs/rfcs/rfc.md"));
    let hits = vec!["docs/other.md".to_string()];

    let receipt = evaluate_gold_query_paths(&query, hits);

    assert!(!receipt.passed);
    assert_eq!(receipt.missing_paths, vec!["docs/rfcs/rfc.md"]);
    assert_eq!(receipt.observed_top_path.as_deref(), Some("docs/other.md"));
    assert_eq!(receipt.best_required_path_rank, None);
    assert_eq!(receipt.required_path_recall_at_1_bps, 0);
    assert_eq!(receipt.required_path_recall_at_3_bps, 0);
    assert_eq!(receipt.required_path_recall_at_5_bps, 0);
    assert_eq!(receipt.required_path_recall_at_10_bps, 0);
    assert_eq!(receipt.mean_required_path_reciprocal_rank_bps, 0);
}

#[test]
fn evaluator_records_late_required_path_rank_quality() {
    let mut query = gold_query(None);
    query.must_hit_paths = vec!["docs/rfcs/rfc.md".to_string()];
    let hits = vec!["docs/other.md".to_string(), "docs/rfcs/rfc.md".to_string()];

    let receipt = evaluate_gold_query_paths(&query, hits);

    assert!(receipt.passed);
    assert_eq!(receipt.best_required_path_rank, Some(1));
    assert_eq!(receipt.required_path_recall_at_1_bps, 0);
    assert_eq!(receipt.required_path_recall_at_3_bps, 10_000);
    assert_eq!(receipt.required_path_recall_at_5_bps, 10_000);
    assert_eq!(receipt.required_path_recall_at_10_bps, 10_000);
    assert_eq!(receipt.mean_required_path_reciprocal_rank_bps, 5_000);
    assert_eq!(
        receipt.required_path_ranks[0].path,
        "docs/rfcs/rfc.md".to_string()
    );
    assert_eq!(receipt.required_path_ranks[0].zero_based_rank, Some(1));
}
