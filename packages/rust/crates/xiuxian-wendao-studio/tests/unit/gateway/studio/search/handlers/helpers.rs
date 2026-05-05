#[cfg(feature = "duckdb")]
use std::fs;
#[cfg(feature = "duckdb")]
use std::path::Path;
use std::sync::Arc;

#[cfg(feature = "duckdb")]
use crate::contracts::{UiConfig, UiProjectConfig};
#[cfg(feature = "duckdb")]
use crate::studio::build_ast_index;
#[cfg(feature = "duckdb")]
use xiuxian_wendao::set_link_graph_wendao_config_override;

pub(crate) fn test_studio_state() -> crate::studio::StudioState {
    let nonce = format!(
        "search-plane-handlers-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|error| panic!("system time before unix epoch: {error}"))
            .as_nanos()
    );
    let search_plane_root = std::env::temp_dir().join(nonce);
    crate::studio::StudioState::new_with_bootstrap_ui_config_and_search_plane_root(
        Arc::new(
            xiuxian_wendao::analyzers::bootstrap_builtin_registry()
                .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
        ),
        search_plane_root,
    )
}

pub(crate) fn test_studio_state_with_cache() -> crate::studio::StudioState {
    let nonce = format!(
        "search-plane-handlers-cache-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|error| panic!("system time before unix epoch: {error}"))
            .as_nanos()
    );
    let project_root = xiuxian_io::PrjDirs::project_root();
    let search_plane_root = std::env::temp_dir().join(nonce);
    let manifest_keyspace = xiuxian_wendao::search::SearchManifestKeyspace::new(format!(
        "xiuxian:test:search_plane:{}",
        blake3::hash(search_plane_root.to_string_lossy().as_bytes()).to_hex()
    ));
    let search_plane = xiuxian_wendao::search::SearchPlaneService::with_test_cache(
        project_root.clone(),
        search_plane_root,
        manifest_keyspace,
        xiuxian_wendao::search::SearchMaintenancePolicy::default(),
    );
    crate::studio::StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane(
        Arc::new(
            xiuxian_wendao::analyzers::bootstrap_builtin_registry()
                .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
        ),
        project_root.clone(),
        project_root,
        search_plane,
    )
}

#[cfg(feature = "duckdb")]
pub(crate) fn configure_local_workspace(studio: &mut crate::studio::StudioState, root: &Path) {
    studio.project_root = root.to_path_buf();
    studio.config_root = root.to_path_buf();
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec![".".to_string()],
        }],
        repo_projects: Vec::new(),
    });
}

#[cfg(feature = "duckdb")]
pub(crate) fn write_search_duckdb_runtime_override(
    body: &str,
) -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(&config_path, body)?;
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());
    Ok(temp)
}

#[cfg(feature = "duckdb")]
pub(crate) async fn publish_local_symbol_index(studio: &crate::studio::StudioState) {
    let projects = studio.configured_projects();
    let hits = build_ast_index(
        studio.project_root.as_path(),
        studio.config_root.as_path(),
        &projects,
    );
    let fingerprint = format!(
        "test:{}",
        blake3::hash(
            format!(
                "{}:{}:{}",
                studio.project_root.display(),
                studio.config_root.display(),
                hits.len()
            )
            .as_bytes()
        )
        .to_hex()
    );
    let hits = crate::contracts::domain_ast_hits_for_search_plane(hits);
    studio
        .search_plane
        .publish_local_symbol_hits(fingerprint.as_str(), hits.as_slice())
        .await
        .unwrap_or_else(|error| panic!("publish local symbol epoch: {error}"));
}

#[cfg(feature = "duckdb")]
pub(crate) async fn publish_knowledge_section_index(studio: &crate::studio::StudioState) {
    let projects = studio.configured_projects();
    let fingerprint = format!(
        "test:knowledge:{}",
        blake3::hash(
            format!(
                "{}:{}:{}",
                studio.project_root.display(),
                studio.config_root.display(),
                projects.len()
            )
            .as_bytes()
        )
        .to_hex()
    );
    studio
        .search_plane
        .publish_knowledge_sections_from_projects(
            studio.project_root.as_path(),
            studio.config_root.as_path(),
            &projects,
            fingerprint.as_str(),
        )
        .await
        .unwrap_or_else(|error| panic!("publish knowledge section epoch: {error}"));
}

pub(crate) async fn publish_repo_content_chunk_index(
    studio: &crate::studio::StudioState,
    repo_id: &str,
    documents: Vec<xiuxian_wendao::repo_index::RepoCodeDocument>,
) {
    studio
        .search_plane
        .publish_repo_content_chunks_with_revision(repo_id, &documents, None)
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks: {error}"));
}

pub(crate) async fn publish_repo_entity_index(
    studio: &crate::studio::StudioState,
    repo_id: &str,
    analysis: &xiuxian_wendao::analyzers::RepositoryAnalysisOutput,
) {
    studio
        .search_plane
        .publish_repo_entities_with_revision(repo_id, analysis, &sample_repo_documents(), None)
        .await
        .unwrap_or_else(|error| panic!("publish repo entities: {error}"));
}

pub(crate) fn sample_repo_analysis(
    repo_id: &str,
) -> xiuxian_wendao::analyzers::RepositoryAnalysisOutput {
    xiuxian_wendao::analyzers::RepositoryAnalysisOutput {
        modules: vec![xiuxian_wendao::analyzers::ModuleRecord {
            repo_id: repo_id.to_string(),
            module_id: "module:BaseModelica".to_string(),
            qualified_name: "BaseModelica".to_string(),
            path: "src/BaseModelica.jl".to_string(),
        }],
        symbols: vec![xiuxian_wendao::analyzers::SymbolRecord {
            repo_id: repo_id.to_string(),
            symbol_id: "symbol:reexport".to_string(),
            module_id: Some("module:BaseModelica".to_string()),
            name: "reexport".to_string(),
            qualified_name: "BaseModelica.reexport".to_string(),
            kind: xiuxian_wendao::analyzers::RepoSymbolKind::Function,
            path: "src/BaseModelica.jl".to_string(),
            line_start: Some(7),
            line_end: Some(9),
            signature: Some("reexport()".to_string()),
            audit_status: Some("verified".to_string()),
            verification_state: Some("verified".to_string()),
            attributes: std::collections::BTreeMap::new(),
        }],
        examples: vec![xiuxian_wendao::analyzers::ExampleRecord {
            repo_id: repo_id.to_string(),
            example_id: "example:reexport".to_string(),
            title: "Reexport example".to_string(),
            path: "examples/reexport.jl".to_string(),
            summary: Some("Shows how to reexport ModelingToolkit".to_string()),
        }],
        ..xiuxian_wendao::analyzers::RepositoryAnalysisOutput::default()
    }
}

pub(crate) fn sample_repo_documents() -> Vec<xiuxian_wendao::repo_index::RepoCodeDocument> {
    vec![
        xiuxian_wendao::repo_index::RepoCodeDocument {
            path: "src/BaseModelica.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from(
                "module BaseModelica\nexport reexport\nreexport() = nothing\nend\n",
            ),
            size_bytes: 61,
            modified_unix_ms: 10,
        },
        xiuxian_wendao::repo_index::RepoCodeDocument {
            path: "examples/reexport.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from("using BaseModelica\nreexport()\n"),
            size_bytes: 29,
            modified_unix_ms: 10,
        },
    ]
}
