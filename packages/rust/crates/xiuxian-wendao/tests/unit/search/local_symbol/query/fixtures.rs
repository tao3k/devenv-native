use std::path::PathBuf;

use crate::search::contracts::{AstSearchHit, StudioNavigationTarget};
use crate::search::{SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService};
#[cfg(feature = "duckdb")]
use crate::set_link_graph_wendao_config_override;
#[cfg(feature = "duckdb")]
use std::fs;

pub(super) fn fixture_service(temp_dir: &tempfile::TempDir) -> SearchPlaneService {
    SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        SearchManifestKeyspace::new("xiuxian:test:local_symbol"),
        SearchMaintenancePolicy::default(),
    )
}

#[cfg(feature = "duckdb")]
pub(super) fn write_search_duckdb_runtime_override(
    body: &str,
) -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(&config_path, body)?;
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());
    Ok(temp)
}

pub(super) fn sample_hit(name: &str, path: &str, line_start: usize) -> AstSearchHit {
    AstSearchHit {
        name: name.to_string(),
        signature: format!("fn {name}()"),
        path: path.to_string().into(),
        language: "rust".to_string(),
        crate_name: "kernel".to_string(),
        project_name: None,
        root_label: None,
        node_kind: None,
        owner_title: None,
        navigation_target: StudioNavigationTarget {
            path: path.to_string().into(),
            category: "symbol".to_string(),
            project_name: None,
            root_label: None,
            line: Some(line_start),
            line_end: Some(line_start),
            column: Some(1),
        },
        line_start,
        line_end: line_start,
        score: 0.0,
    }
}

pub(super) fn sample_markdown_hit(
    name: &str,
    node_kind: Option<&str>,
    owner_title: Option<&str>,
) -> AstSearchHit {
    AstSearchHit {
        name: name.to_string(),
        signature: format!("## {name}"),
        path: "docs/alpha.md".to_string().into(),
        language: "markdown".to_string(),
        crate_name: "docs".to_string(),
        project_name: None,
        root_label: None,
        node_kind: node_kind.map(ToOwned::to_owned),
        owner_title: owner_title.map(ToOwned::to_owned),
        navigation_target: StudioNavigationTarget {
            path: "docs/alpha.md".to_string().into(),
            category: "symbol".to_string(),
            project_name: None,
            root_label: None,
            line: Some(1),
            line_end: Some(1),
            column: Some(1),
        },
        line_start: 1,
        line_end: 1,
        score: 0.0,
    }
}

pub(super) async fn publish_local_symbol_hits(
    service: &SearchPlaneService,
    build_id: &str,
    hits: &[AstSearchHit],
) {
    crate::search::local_symbol::build::publish_local_symbol_hits(service, build_id, hits)
        .await
        .unwrap_or_else(|error| panic!("publish local symbol hits: {error}"));
}
