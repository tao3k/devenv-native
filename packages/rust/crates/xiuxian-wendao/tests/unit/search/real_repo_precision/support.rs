use std::path::PathBuf;

use crate::search::real_repo_precision::{RealRepoGoldQuery, RealRepoGoldQueryKind};

pub(super) fn gold_query(required_top_path: Option<&str>) -> RealRepoGoldQuery {
    RealRepoGoldQuery {
        id: "gold".to_string(),
        kind: RealRepoGoldQueryKind::LinkGraph,
        query: "semantic SSOT".to_string(),
        limit: 5,
        must_hit_paths: vec!["docs/rfcs/rfc.md".to_string()],
        required_top_path: required_top_path.map(str::to_string),
        language_filters: Vec::new(),
    }
}

pub(super) fn test_project_root() -> PathBuf {
    std::env::var_os("PRJ_ROOT").map_or_else(
        || {
            let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            for _ in 0..4 {
                root = root
                    .parent()
                    .unwrap_or_else(|| panic!("crate path should be under the repository root"))
                    .to_path_buf();
            }
            root
        },
        PathBuf::from,
    )
}
