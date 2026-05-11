use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::analyzers::{
    ExampleRecord, ImportKind, ImportRecord, ModuleRecord, RepoSymbolKind,
    RepositoryAnalysisOutput, SymbolRecord,
};
use crate::repo_index::RepoCodeDocument;
use crate::search::repo_entity::publish_repo_entities;
use crate::search::{SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService};

pub(super) struct RepoEntityQueryFixture {
    pub(super) _temp_dir: tempfile::TempDir,
    pub(super) service: SearchPlaneService,
}

pub(super) async fn published_repo_entity_fixture(
    repo_id: &str,
    symbol_name: &str,
    example_summary: &str,
) -> RepoEntityQueryFixture {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let service = SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        SearchManifestKeyspace::new("xiuxian:test:repo-entity-query"),
        SearchMaintenancePolicy::default(),
    );
    let analysis = sample_analysis(repo_id, symbol_name, example_summary);
    let documents = sample_documents(symbol_name, 10);
    publish_repo_entities(&service, repo_id, &analysis, &documents, Some("rev-1"))
        .await
        .unwrap_or_else(|error| panic!("publish repo entities: {error}"));
    RepoEntityQueryFixture {
        _temp_dir: temp_dir,
        service,
    }
}

fn sample_analysis(
    repo_id: &str,
    symbol_name: &str,
    example_summary: &str,
) -> RepositoryAnalysisOutput {
    let mut attributes = BTreeMap::new();
    attributes.insert("arity".to_string(), "0".to_string());
    RepositoryAnalysisOutput {
        modules: vec![ModuleRecord {
            repo_id: repo_id.to_string().into(),
            module_id: "module:BaseModelica".to_string().into(),
            qualified_name: "BaseModelica".to_string(),
            path: "src/BaseModelica.jl".to_string().into(),
        }],
        symbols: vec![SymbolRecord {
            repo_id: repo_id.to_string().into(),
            symbol_id: format!("symbol:{symbol_name}").into(),
            module_id: Some("module:BaseModelica".to_string().into()),
            name: symbol_name.to_string(),
            qualified_name: format!("BaseModelica.{symbol_name}"),
            kind: RepoSymbolKind::Function,
            path: "src/BaseModelica.jl".to_string().into(),
            line_start: Some(7),
            line_end: Some(9),
            signature: Some(format!("{symbol_name}()")),
            audit_status: Some("verified".to_string().into()),
            verification_state: Some("verified".to_string().into()),
            attributes,
        }],
        examples: vec![ExampleRecord {
            repo_id: repo_id.to_string().into(),
            example_id: "example:solve".to_string().into(),
            title: "Solve example".to_string(),
            path: "examples/solve.jl".to_string().into(),
            summary: Some(example_summary.to_string()),
        }],
        imports: vec![ImportRecord {
            repo_id: repo_id.to_string().into(),
            module_id: "module:BaseModelica".to_string().into(),
            path: "src/BaseModelica.jl".to_string().into(),
            import_name: "solve".to_string(),
            target_package: "SciMLBase".to_string(),
            source_module: "BaseModelica".to_string(),
            kind: ImportKind::Reexport,
            line_start: None,
            resolved_id: Some(format!("symbol:{symbol_name}").into()),
            attributes: BTreeMap::new(),
        }],
        ..RepositoryAnalysisOutput::default()
    }
}

fn sample_documents(symbol_name: &str, source_modified_unix_ms: u64) -> Vec<RepoCodeDocument> {
    vec![
        RepoCodeDocument {
            path: "src/BaseModelica.jl".to_string().into(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from(format!(
                "module BaseModelica\n{symbol_name}() = nothing\nend\n"
            )),
            size_bytes: 48,
            modified_unix_ms: source_modified_unix_ms,
        },
        RepoCodeDocument {
            path: "examples/solve.jl".to_string().into(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from("using BaseModelica\nsolve()\n"),
            size_bytes: 28,
            modified_unix_ms: 10,
        },
    ]
}
