use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use tempfile::Builder;

use crate::analyzers::{
    ExampleRecord, ModuleRecord, RepoSymbolKind, RepositoryAnalysisOutput, SymbolRecord,
};
use crate::query_core::context::{GraphBackend, RetrievalBackend};
use crate::query_core::operators::{GraphNeighborsOp, PayloadFetchOp, VectorSearchOp};
use crate::query_core::{WendaoQueryCoreError, WendaoRelation};
use crate::repo_index::RepoCodeDocument;

pub(super) fn tempdir_or_panic(context: &str) -> tempfile::TempDir {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".cache")
        .join("query-core-tests");
    std::fs::create_dir_all(&root)
        .unwrap_or_else(|error| panic!("create query-core test temp root: {error}"));
    Builder::new()
        .prefix("query-core-")
        .tempdir_in(root)
        .unwrap_or_else(|error| panic!("{context}: {error}"))
}

pub(super) fn write_fixture(path: &Path, contents: &str, context: &str) {
    std::fs::write(path, contents).unwrap_or_else(|error| panic!("{context}: {error}"));
}

pub(super) fn repo_document(
    path: &str,
    contents: &str,
    size_bytes: u64,
    modified_unix_ms: u64,
) -> RepoCodeDocument {
    RepoCodeDocument {
        path: path.to_string(),
        language: Some("rust".to_string()),
        contents: Arc::<str>::from(contents),
        size_bytes,
        modified_unix_ms,
    }
}

pub(super) fn sample_repo_analysis(repo_id: &str) -> RepositoryAnalysisOutput {
    RepositoryAnalysisOutput {
        modules: vec![ModuleRecord {
            repo_id: repo_id.to_string(),
            module_id: "module:BaseModelica".to_string(),
            qualified_name: "BaseModelica".to_string(),
            path: "src/BaseModelica.jl".to_string(),
        }],
        symbols: vec![SymbolRecord {
            repo_id: repo_id.to_string(),
            symbol_id: "symbol:reexport".to_string(),
            module_id: Some("module:BaseModelica".to_string()),
            name: "reexport".to_string(),
            qualified_name: "BaseModelica.reexport".to_string(),
            kind: RepoSymbolKind::Function,
            path: "src/BaseModelica.jl".to_string(),
            line_start: Some(7),
            line_end: Some(9),
            signature: Some("reexport()".to_string()),
            audit_status: Some("verified".to_string()),
            verification_state: Some("verified".to_string()),
            attributes: std::collections::BTreeMap::new(),
        }],
        examples: vec![ExampleRecord {
            repo_id: repo_id.to_string(),
            example_id: "example:reexport".to_string(),
            title: "Reexport example".to_string(),
            path: "examples/reexport.jl".to_string(),
            summary: Some("Shows how to reexport ModelingToolkit".to_string()),
        }],
        ..RepositoryAnalysisOutput::default()
    }
}

pub(super) fn sample_repo_documents() -> Vec<RepoCodeDocument> {
    vec![
        repo_document(
            "src/BaseModelica.jl",
            "module BaseModelica\nexport reexport\nreexport() = nothing\nend\n",
            61,
            10,
        ),
        RepoCodeDocument {
            path: "examples/reexport.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from("using BaseModelica\nreexport()\n"),
            size_bytes: 29,
            modified_unix_ms: 10,
        },
    ]
}

pub(super) fn snapshot_retrieval_rows(relation: &WendaoRelation) -> Vec<serde_json::Value> {
    relation
        .batches()
        .iter()
        .flat_map(|batch| {
            xiuxian_db_store::retrieval_rows_from_record_batch(batch)
                .unwrap_or_else(|error| panic!("decode retrieval rows: {error}"))
                .into_iter()
                .map(|row| {
                    serde_json::json!({
                        "id": row.id,
                        "path": row.path,
                        "repo": row.repo,
                        "title": row.title,
                        "score": row.score.map(crate::gateway::studio::test_support::round_f64),
                        "source": row.source,
                        "snippet": row.snippet,
                        "doc_type": row.doc_type,
                        "match_reason": row.match_reason,
                        "best_section": row.best_section,
                        "language": row.language,
                        "line": row.line,
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

pub(super) struct StubGraphBackend {
    pub(super) relation: WendaoRelation,
}

#[async_trait]
impl GraphBackend for StubGraphBackend {
    async fn graph_neighbors(
        &self,
        _op: &GraphNeighborsOp,
    ) -> Result<WendaoRelation, WendaoQueryCoreError> {
        Ok(self.relation.clone())
    }
}

pub(super) struct StubPayloadRetrievalBackend;

#[async_trait]
impl RetrievalBackend for StubPayloadRetrievalBackend {
    async fn vector_search(
        &self,
        _op: &VectorSearchOp,
    ) -> Result<WendaoRelation, WendaoQueryCoreError> {
        Err(WendaoQueryCoreError::Backend(
            "stub payload backend does not implement vector_search".to_string(),
        ))
    }

    async fn payload_fetch(
        &self,
        relation: &WendaoRelation,
        op: &PayloadFetchOp,
    ) -> Result<WendaoRelation, WendaoQueryCoreError> {
        let batches = relation
            .batches()
            .iter()
            .map(|batch| {
                xiuxian_db_store::payload_fetch_record_batch(batch, &op.columns, op.ids.as_ref())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let schema = batches
            .first()
            .map(xiuxian_db_store::EngineRecordBatch::schema)
            .ok_or_else(|| WendaoQueryCoreError::InvalidRelation("missing payload batch".into()))?;
        Ok(WendaoRelation::new(schema, batches))
    }
}

pub(super) fn temp_project_root() -> PathBuf {
    PathBuf::from("/tmp/project")
}
