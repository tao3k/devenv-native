use std::path::PathBuf;

#[cfg(feature = "duckdb")]
use std::fs;

use crate::search::cache::SearchPlaneCache;
use crate::search::contracts::SearchProjectConfig;
use crate::search::knowledge_section::build::publish_knowledge_sections_from_projects;
use crate::search::{SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService};
#[cfg(feature = "duckdb")]
use crate::set_link_graph_wendao_config_override;

pub(super) struct KnowledgeFixture {
    pub(super) service: SearchPlaneService,
    pub(super) project_root: PathBuf,
}

pub(super) fn fixture_service(temp_dir: &tempfile::TempDir) -> KnowledgeFixture {
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    let keyspace = SearchManifestKeyspace::new("xiuxian:test:knowledge-query");
    let cache = SearchPlaneCache::for_tests(keyspace.clone());
    let service = SearchPlaneService::with_runtime(
        project_root.clone(),
        storage_root,
        keyspace,
        SearchMaintenancePolicy::default(),
        cache,
    );
    KnowledgeFixture {
        service,
        project_root,
    }
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

pub(super) async fn publish_knowledge_notes(
    fixture: &KnowledgeFixture,
    build_id: &str,
    notes: &[(&str, &str)],
) {
    std::fs::create_dir_all(&fixture.project_root)
        .unwrap_or_else(|error| panic!("create workspace root: {error}"));
    for (path, body) in notes {
        let note_path = fixture.project_root.join(path);
        if let Some(parent) = note_path.parent() {
            std::fs::create_dir_all(parent)
                .unwrap_or_else(|error| panic!("create note parent: {error}"));
        }
        std::fs::write(&note_path, body).unwrap_or_else(|error| panic!("write note: {error}"));
    }

    let projects = vec![SearchProjectConfig {
        name: "notes".to_string(),
        root: ".".to_string(),
        dirs: vec![".".to_string()],
    }];
    publish_knowledge_sections_from_projects(
        &fixture.service,
        fixture.project_root.as_path(),
        fixture.project_root.as_path(),
        &projects,
        build_id,
    )
    .await
    .unwrap_or_else(|error| panic!("publish knowledge sections: {error}"));
}
