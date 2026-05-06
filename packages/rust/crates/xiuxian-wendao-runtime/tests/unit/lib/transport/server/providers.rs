use std::sync::{Arc, Mutex};

use arrow_array::{
    Int32Array as ArrowInt32Array, Int64Array as ArrowInt64Array, RecordBatch as ArrowRecordBatch,
    StringArray as ArrowStringArray,
};
use arrow_schema::{DataType as ArrowDataType, Field as ArrowField, Schema as ArrowSchema};
use async_trait::async_trait;
use xiuxian_db_store::{
    LanceBooleanArray, LanceDataType, LanceField, LanceFloat64Array, LanceInt32Array,
    LanceRecordBatch, LanceSchema, LanceStringArray as StringArray,
};

use crate::transport::{
    AnalysisFlightRouteResponse, AstSearchFlightRouteProvider, AttachmentSearchFlightRouteProvider,
    AutocompleteFlightRouteProvider, AutocompleteFlightRouteResponse,
    CodeAstAnalysisFlightRouteProvider, DefinitionFlightRouteProvider,
    DefinitionFlightRouteResponse, DocumentExtractFlightRouteProvider,
    DocumentExtractFlightRouteResponse, GraphNeighborsFlightRouteProvider,
    GraphNeighborsFlightRouteResponse, MarkdownAnalysisFlightRouteProvider,
    RepoDocCoverageFlightRouteProvider, RepoIndexStatusFlightRouteProvider,
    RepoOverviewFlightRouteProvider, RepoSearchFlightRequest, RepoSearchFlightRouteProvider,
    RepoSyncFlightRouteProvider, SearchFlightRouteProvider, SearchFlightRouteResponse,
    SqlFlightRouteProvider, SqlFlightRouteResponse, VfsContentFlightRouteProvider,
    VfsContentFlightRouteResponse, VfsResolveFlightRouteProvider, VfsResolveFlightRouteResponse,
};

type SearchRequestRecord = (String, String, usize, Option<String>, Option<String>);
type DefinitionRequestRecord = (String, Option<String>, Option<usize>);
type AutocompleteRequestRecord = (String, usize);
type GraphNeighborsRequestRecord = (String, String, usize, usize);
type AttachmentSearchRequestRecord = (String, usize, Vec<String>, Vec<String>, bool);
type AstSearchRequestRecord = (String, usize);
type CodeAstAnalysisRequestRecord = (String, String, Option<usize>);
type RepoOverviewRequestRecord = String;
type RepoIndexStatusRequestRecord = Option<String>;
type RepoSyncRequestRecord = (String, String);
type RepoDocCoverageRequestRecord = (String, Option<String>);
type DocumentExtractRequestRecord = (String, String, bool, bool, String);
type VfsContentRequestRecord = String;

fn lock_or_panic<'a, T>(mutex: &'a Mutex<T>, context: &str) -> std::sync::MutexGuard<'a, T> {
    mutex.lock().unwrap_or_else(|_| panic!("{context}"))
}

#[derive(Debug)]
pub(super) struct RecordingRepoSearchProvider;

#[async_trait]
impl RepoSearchFlightRouteProvider for RecordingRepoSearchProvider {
    async fn repo_search_batch(
        &self,
        request: &RepoSearchFlightRequest,
    ) -> Result<LanceRecordBatch, String> {
        LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("doc_id", LanceDataType::Utf8, false),
                LanceField::new("path", LanceDataType::Utf8, false),
                LanceField::new("title", LanceDataType::Utf8, false),
                LanceField::new("score", LanceDataType::Float64, false),
                LanceField::new("language", LanceDataType::Utf8, false),
            ])),
            vec![
                Arc::new(StringArray::from(vec![format!(
                    "doc:{}:{}",
                    request.query_text, request.limit
                )])),
                Arc::new(StringArray::from(vec!["src/lib.rs"])),
                Arc::new(StringArray::from(vec!["Repo Search Result"])),
                Arc::new(LanceFloat64Array::from(vec![0.91_f64])),
                Arc::new(StringArray::from(vec!["rust"])),
            ],
        )
        .map_err(|error| error.to_string())
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingSearchProvider {
    request: Mutex<Option<SearchRequestRecord>>,
    call_count: Mutex<usize>,
}

impl RecordingSearchProvider {
    pub(super) fn recorded_request(&self) -> Option<SearchRequestRecord> {
        lock_or_panic(&self.request, "search-family provider record should lock").clone()
    }

    pub(super) fn call_count(&self) -> usize {
        *lock_or_panic(
            &self.call_count,
            "search-family provider call count should lock",
        )
    }
}

#[async_trait]
impl SearchFlightRouteProvider for RecordingSearchProvider {
    async fn search_batch(
        &self,
        route: &str,
        query_text: &str,
        limit: usize,
        intent: Option<&str>,
        repo_hint: Option<&str>,
    ) -> Result<SearchFlightRouteResponse, String> {
        *lock_or_panic(&self.request, "search-family provider record should lock") = Some((
            route.to_string(),
            query_text.to_string(),
            limit,
            intent.map(ToString::to_string),
            repo_hint.map(ToString::to_string),
        ));
        *lock_or_panic(
            &self.call_count,
            "search-family provider call count should lock",
        ) += 1;
        let batch = LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("doc_id", LanceDataType::Utf8, false),
                LanceField::new("route", LanceDataType::Utf8, false),
                LanceField::new("query_text", LanceDataType::Utf8, false),
                LanceField::new("score", LanceDataType::Float64, false),
            ])),
            vec![
                Arc::new(StringArray::from(vec![format!(
                    "{route}:{query_text}:{limit}"
                )])),
                Arc::new(StringArray::from(vec![route.to_string()])),
                Arc::new(StringArray::from(vec![query_text.to_string()])),
                Arc::new(LanceFloat64Array::from(vec![0.99_f64])),
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(SearchFlightRouteResponse::new(batch).with_app_metadata(
            serde_json::json!({
                "query": query_text,
                "hitCount": 1,
                "selectedMode": route,
                "intent": intent,
                "repoHint": repo_hint,
            })
            .to_string()
            .into_bytes(),
        ))
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingDefinitionProvider {
    request: Mutex<Option<DefinitionRequestRecord>>,
    call_count: Mutex<usize>,
}

impl RecordingDefinitionProvider {
    pub(super) fn recorded_request(&self) -> Option<DefinitionRequestRecord> {
        lock_or_panic(&self.request, "definition provider record should lock").clone()
    }

    pub(super) fn call_count(&self) -> usize {
        *lock_or_panic(
            &self.call_count,
            "definition provider call count should lock",
        )
    }
}

#[async_trait]
impl DefinitionFlightRouteProvider for RecordingDefinitionProvider {
    async fn definition_batch(
        &self,
        query_text: &str,
        source_path: Option<&str>,
        source_line: Option<usize>,
    ) -> Result<DefinitionFlightRouteResponse, tonic::Status> {
        *lock_or_panic(&self.request, "definition provider record should lock") = Some((
            query_text.to_string(),
            source_path.map(ToString::to_string),
            source_line,
        ));
        *lock_or_panic(
            &self.call_count,
            "definition provider call count should lock",
        ) += 1;
        let batch = LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("name", LanceDataType::Utf8, false),
                LanceField::new("path", LanceDataType::Utf8, false),
            ])),
            vec![
                Arc::new(StringArray::from(vec![query_text.to_string()])),
                Arc::new(StringArray::from(vec![
                    source_path.unwrap_or("src/lib.rs").to_string(),
                ])),
            ],
        )
        .map_err(|error| tonic::Status::internal(error.to_string()))?;
        Ok(DefinitionFlightRouteResponse::new(batch).with_app_metadata(
            serde_json::json!({
                "query": query_text,
                "sourcePath": source_path,
                "sourceLine": source_line,
                "candidateCount": 1,
                "selectedScope": "definition",
            })
            .to_string()
            .into_bytes(),
        ))
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingAutocompleteProvider {
    request: Mutex<Option<AutocompleteRequestRecord>>,
    call_count: Mutex<usize>,
}

impl RecordingAutocompleteProvider {
    pub(super) fn recorded_request(&self) -> Option<AutocompleteRequestRecord> {
        lock_or_panic(&self.request, "autocomplete provider record should lock").clone()
    }

    pub(super) fn call_count(&self) -> usize {
        *lock_or_panic(
            &self.call_count,
            "autocomplete provider call count should lock",
        )
    }
}

#[async_trait]
impl AutocompleteFlightRouteProvider for RecordingAutocompleteProvider {
    async fn autocomplete_batch(
        &self,
        prefix: &str,
        limit: usize,
    ) -> Result<AutocompleteFlightRouteResponse, tonic::Status> {
        *lock_or_panic(&self.request, "autocomplete provider record should lock") =
            Some((prefix.to_string(), limit));
        *lock_or_panic(
            &self.call_count,
            "autocomplete provider call count should lock",
        ) += 1;
        let batch = LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("text", LanceDataType::Utf8, false),
                LanceField::new("suggestionType", LanceDataType::Utf8, false),
            ])),
            vec![
                Arc::new(StringArray::from(vec![format!("{prefix}_suggestion")])),
                Arc::new(StringArray::from(vec!["symbol"])),
            ],
        )
        .map_err(|error| tonic::Status::internal(error.to_string()))?;
        Ok(
            AutocompleteFlightRouteResponse::new(batch).with_app_metadata(
                serde_json::json!({
                    "prefix": prefix,
                })
                .to_string()
                .into_bytes(),
            ),
        )
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingSqlProvider {
    request: Mutex<Option<String>>,
    call_count: Mutex<usize>,
}

impl RecordingSqlProvider {
    pub(super) fn call_count(&self) -> usize {
        *lock_or_panic(&self.call_count, "SQL provider call count should lock")
    }
}

#[async_trait]
impl SqlFlightRouteProvider for RecordingSqlProvider {
    async fn sql_query_batches(&self, query_text: &str) -> Result<SqlFlightRouteResponse, String> {
        *lock_or_panic(&self.request, "SQL provider record should lock") =
            Some(query_text.to_string());
        *lock_or_panic(&self.call_count, "SQL provider call count should lock") += 1;
        let schema = Arc::new(ArrowSchema::new(vec![
            ArrowField::new("table_name", ArrowDataType::Utf8, false),
            ArrowField::new("row_id", ArrowDataType::Int32, false),
        ]));
        let first_batch = ArrowRecordBatch::try_new(
            Arc::clone(&schema),
            vec![
                Arc::new(ArrowStringArray::from(vec!["repo_entity"])),
                Arc::new(ArrowInt32Array::from(vec![1])),
            ],
        )
        .map_err(|error| error.to_string())?;
        let second_batch = ArrowRecordBatch::try_new(
            schema,
            vec![
                Arc::new(ArrowStringArray::from(vec!["repo_content_chunk"])),
                Arc::new(ArrowInt32Array::from(vec![2])),
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(
            SqlFlightRouteResponse::new(vec![first_batch, second_batch]).with_app_metadata(
                serde_json::json!({
                    "query": query_text,
                    "batchCount": 2,
                    "registeredTables": ["repo_entity", "repo_content_chunk"],
                })
                .to_string()
                .into_bytes(),
            ),
        )
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingVfsResolveProvider {
    request: Mutex<Option<String>>,
    call_count: Mutex<usize>,
}

impl RecordingVfsResolveProvider {
    pub(super) fn recorded_request(&self) -> Option<String> {
        lock_or_panic(&self.request, "VFS resolve provider record should lock").clone()
    }

    pub(super) fn call_count(&self) -> usize {
        *lock_or_panic(
            &self.call_count,
            "VFS resolve provider call count should lock",
        )
    }
}

#[async_trait]
impl VfsResolveFlightRouteProvider for RecordingVfsResolveProvider {
    async fn resolve_vfs_navigation_batch(
        &self,
        path: &str,
    ) -> Result<VfsResolveFlightRouteResponse, tonic::Status> {
        *lock_or_panic(&self.request, "VFS resolve provider record should lock") =
            Some(path.to_string());
        *lock_or_panic(
            &self.call_count,
            "VFS resolve provider call count should lock",
        ) += 1;
        let batch = LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("path", LanceDataType::Utf8, false),
                LanceField::new("category", LanceDataType::Utf8, false),
            ])),
            vec![
                Arc::new(StringArray::from(vec![path.to_string()])),
                Arc::new(StringArray::from(vec!["file".to_string()])),
            ],
        )
        .map_err(|error| tonic::Status::internal(error.to_string()))?;
        Ok(VfsResolveFlightRouteResponse::new(batch).with_app_metadata(
            serde_json::json!({
                "path": path,
                "navigationTarget": {
                    "path": path,
                    "category": "file",
                },
            })
            .to_string()
            .into_bytes(),
        ))
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingVfsContentProvider {
    request: Mutex<Option<VfsContentRequestRecord>>,
    call_count: Mutex<usize>,
}

impl RecordingVfsContentProvider {
    pub(super) fn recorded_request(&self) -> Option<VfsContentRequestRecord> {
        lock_or_panic(&self.request, "VFS content provider record should lock").clone()
    }

    pub(super) fn call_count(&self) -> usize {
        *lock_or_panic(
            &self.call_count,
            "VFS content provider call count should lock",
        )
    }
}

#[async_trait]
impl VfsContentFlightRouteProvider for RecordingVfsContentProvider {
    async fn read_vfs_content_batch(
        &self,
        path: &str,
    ) -> Result<VfsContentFlightRouteResponse, tonic::Status> {
        *lock_or_panic(&self.request, "VFS content provider record should lock") =
            Some(path.to_string());
        *lock_or_panic(
            &self.call_count,
            "VFS content provider call count should lock",
        ) += 1;
        let batch = LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("path", LanceDataType::Utf8, false),
                LanceField::new("contentType", LanceDataType::Utf8, false),
                LanceField::new("content", LanceDataType::Utf8, false),
                LanceField::new("modified", LanceDataType::Int32, false),
            ])),
            vec![
                Arc::new(StringArray::from(vec![path.to_string()])),
                Arc::new(StringArray::from(vec!["text/plain".to_string()])),
                Arc::new(StringArray::from(vec![format!("content:{path}")])),
                Arc::new(LanceInt32Array::from(vec![7])),
            ],
        )
        .map_err(|error| tonic::Status::internal(error.to_string()))?;
        Ok(VfsContentFlightRouteResponse::new(batch).with_app_metadata(
            serde_json::json!({
                "path": path,
                "contentType": "text/plain",
                "modified": 7,
            })
            .to_string()
            .into_bytes(),
        ))
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingGraphNeighborsProvider {
    request: Mutex<Option<GraphNeighborsRequestRecord>>,
    call_count: Mutex<usize>,
}

impl RecordingGraphNeighborsProvider {
    pub(super) fn recorded_request(&self) -> Option<GraphNeighborsRequestRecord> {
        lock_or_panic(&self.request, "graph-neighbors provider record should lock").clone()
    }

    pub(super) fn call_count(&self) -> usize {
        *lock_or_panic(
            &self.call_count,
            "graph-neighbors provider call count should lock",
        )
    }
}

#[async_trait]
impl GraphNeighborsFlightRouteProvider for RecordingGraphNeighborsProvider {
    async fn graph_neighbors_batch(
        &self,
        node_id: &str,
        direction: &str,
        hops: usize,
        limit: usize,
    ) -> Result<GraphNeighborsFlightRouteResponse, tonic::Status> {
        *lock_or_panic(&self.request, "graph-neighbors provider record should lock") =
            Some((node_id.to_string(), direction.to_string(), hops, limit));
        *lock_or_panic(
            &self.call_count,
            "graph-neighbors provider call count should lock",
        ) += 1;
        let batch = LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("rowType", LanceDataType::Utf8, false),
                LanceField::new("nodeId", LanceDataType::Utf8, true),
                LanceField::new("nodeLabel", LanceDataType::Utf8, true),
                LanceField::new("nodePath", LanceDataType::Utf8, true),
                LanceField::new("nodeType", LanceDataType::Utf8, true),
                LanceField::new("nodeIsCenter", LanceDataType::Boolean, true),
                LanceField::new("nodeDistance", LanceDataType::Int32, true),
                LanceField::new("navigationPath", LanceDataType::Utf8, true),
                LanceField::new("navigationCategory", LanceDataType::Utf8, true),
                LanceField::new("navigationProjectName", LanceDataType::Utf8, true),
                LanceField::new("navigationRootLabel", LanceDataType::Utf8, true),
                LanceField::new("navigationLine", LanceDataType::Int32, true),
                LanceField::new("navigationLineEnd", LanceDataType::Int32, true),
                LanceField::new("navigationColumn", LanceDataType::Int32, true),
                LanceField::new("linkSource", LanceDataType::Utf8, true),
                LanceField::new("linkTarget", LanceDataType::Utf8, true),
                LanceField::new("linkDirection", LanceDataType::Utf8, true),
                LanceField::new("linkDistance", LanceDataType::Int32, true),
            ])),
            vec![
                Arc::new(StringArray::from(vec!["node", "link"])),
                Arc::new(StringArray::from(vec![Some(node_id.to_string()), None])),
                Arc::new(StringArray::from(vec![Some("Index".to_string()), None])),
                Arc::new(StringArray::from(vec![Some(node_id.to_string()), None])),
                Arc::new(StringArray::from(vec![Some("doc".to_string()), None])),
                Arc::new(LanceBooleanArray::from(vec![Some(true), None])),
                Arc::new(LanceInt32Array::from(vec![Some(0), None])),
                Arc::new(StringArray::from(vec![Some(node_id.to_string()), None])),
                Arc::new(StringArray::from(vec![Some("doc".to_string()), None])),
                Arc::new(StringArray::from(vec![Some("kernel".to_string()), None])),
                Arc::new(StringArray::from(vec![Some("project".to_string()), None])),
                Arc::new(LanceInt32Array::from(vec![Some(7), None])),
                Arc::new(LanceInt32Array::from(vec![Some(9), None])),
                Arc::new(LanceInt32Array::from(vec![Some(3), None])),
                Arc::new(StringArray::from(vec![None, Some(node_id.to_string())])),
                Arc::new(StringArray::from(vec![
                    None,
                    Some(format!("{node_id}::neighbor")),
                ])),
                Arc::new(StringArray::from(vec![None, Some(direction.to_string())])),
                Arc::new(LanceInt32Array::from(vec![
                    None,
                    Some(i32::try_from(hops.min(limit)).unwrap_or(i32::MAX)),
                ])),
            ],
        )
        .map_err(|error| tonic::Status::internal(error.to_string()))?;
        Ok(GraphNeighborsFlightRouteResponse::new(batch))
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingAttachmentSearchProvider {
    request: Mutex<Option<AttachmentSearchRequestRecord>>,
}

impl RecordingAttachmentSearchProvider {
    pub(super) fn recorded_request(&self) -> Option<AttachmentSearchRequestRecord> {
        lock_or_panic(
            &self.request,
            "attachment-search provider record should lock",
        )
        .clone()
    }
}

#[async_trait]
impl AttachmentSearchFlightRouteProvider for RecordingAttachmentSearchProvider {
    async fn attachment_search_batch(
        &self,
        query_text: &str,
        limit: usize,
        ext_filters: &std::collections::HashSet<String>,
        kind_filters: &std::collections::HashSet<String>,
        case_sensitive: bool,
    ) -> Result<SearchFlightRouteResponse, String> {
        let mut ext_filters = ext_filters.iter().cloned().collect::<Vec<_>>();
        ext_filters.sort();
        let mut kind_filters = kind_filters.iter().cloned().collect::<Vec<_>>();
        kind_filters.sort();
        *lock_or_panic(
            &self.request,
            "attachment-search provider record should lock",
        ) = Some((
            query_text.to_string(),
            limit,
            ext_filters,
            kind_filters,
            case_sensitive,
        ));
        let batch = LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("doc_id", LanceDataType::Utf8, false),
                LanceField::new("query_text", LanceDataType::Utf8, false),
                LanceField::new("score", LanceDataType::Float64, false),
            ])),
            vec![
                Arc::new(StringArray::from(vec![format!(
                    "attachment:{query_text}:{limit}"
                )])),
                Arc::new(StringArray::from(vec![query_text.to_string()])),
                Arc::new(LanceFloat64Array::from(vec![0.77_f64])),
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(SearchFlightRouteResponse::new(batch).with_app_metadata(
            serde_json::json!({
                "query": query_text,
                "hitCount": 1,
                "selectedScope": "attachments",
                "partial": false,
            })
            .to_string()
            .into_bytes(),
        ))
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingAstSearchProvider {
    request: Mutex<Option<AstSearchRequestRecord>>,
}

impl RecordingAstSearchProvider {
    pub(super) fn recorded_request(&self) -> Option<AstSearchRequestRecord> {
        lock_or_panic(&self.request, "AST-search provider record should lock").clone()
    }
}

#[async_trait]
impl AstSearchFlightRouteProvider for RecordingAstSearchProvider {
    async fn ast_search_batch(
        &self,
        query_text: &str,
        limit: usize,
    ) -> Result<SearchFlightRouteResponse, String> {
        *lock_or_panic(&self.request, "AST-search provider record should lock") =
            Some((query_text.to_string(), limit));
        let batch = LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("doc_id", LanceDataType::Utf8, false),
                LanceField::new("query_text", LanceDataType::Utf8, false),
                LanceField::new("score", LanceDataType::Float64, false),
            ])),
            vec![
                Arc::new(StringArray::from(vec![format!("ast:{query_text}:{limit}")])),
                Arc::new(StringArray::from(vec![query_text.to_string()])),
                Arc::new(LanceFloat64Array::from(vec![0.81_f64])),
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(SearchFlightRouteResponse::new(batch).with_app_metadata(
            serde_json::json!({
                "query": query_text,
                "hitCount": 1,
                "selectedScope": "definitions",
                "partial": false,
            })
            .to_string()
            .into_bytes(),
        ))
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingMarkdownAnalysisProvider {
    request: Mutex<Option<String>>,
    call_count: Mutex<usize>,
}

impl RecordingMarkdownAnalysisProvider {
    pub(super) fn recorded_request(&self) -> Option<String> {
        lock_or_panic(
            &self.request,
            "markdown analysis provider record should lock",
        )
        .clone()
    }

    pub(super) fn call_count(&self) -> usize {
        *lock_or_panic(
            &self.call_count,
            "markdown analysis provider call-count should lock",
        )
    }
}

#[async_trait]
impl MarkdownAnalysisFlightRouteProvider for RecordingMarkdownAnalysisProvider {
    async fn markdown_analysis_batch(
        &self,
        path: &str,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        *lock_or_panic(
            &self.call_count,
            "markdown analysis provider call-count should lock",
        ) += 1;
        *lock_or_panic(
            &self.request,
            "markdown analysis provider record should lock",
        ) = Some(path.to_string());
        let batch = LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("ownerId", LanceDataType::Utf8, false),
                LanceField::new("chunkId", LanceDataType::Utf8, false),
                LanceField::new("semanticType", LanceDataType::Utf8, false),
            ])),
            vec![
                Arc::new(StringArray::from(vec![format!("markdown:{path}")])),
                Arc::new(StringArray::from(vec!["chunk:0"])),
                Arc::new(StringArray::from(vec!["section"])),
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(
            serde_json::json!({
                "path": path,
                "documentHash": "fp:markdown",
                "nodeCount": 1,
                "edgeCount": 0,
                "nodes": [],
                "edges": [],
                "projections": [],
                "diagnostics": [],
            })
            .to_string()
            .into_bytes(),
        ))
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingCodeAstAnalysisProvider {
    request: Mutex<Option<CodeAstAnalysisRequestRecord>>,
}

impl RecordingCodeAstAnalysisProvider {
    pub(super) fn recorded_request(&self) -> Option<CodeAstAnalysisRequestRecord> {
        lock_or_panic(
            &self.request,
            "code-AST analysis provider record should lock",
        )
        .clone()
    }
}

#[async_trait]
impl CodeAstAnalysisFlightRouteProvider for RecordingCodeAstAnalysisProvider {
    async fn code_ast_analysis_batch(
        &self,
        path: &str,
        repo_id: &str,
        line_hint: Option<usize>,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        *lock_or_panic(
            &self.request,
            "code-AST analysis provider record should lock",
        ) = Some((path.to_string(), repo_id.to_string(), line_hint));
        let batch = LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("ownerId", LanceDataType::Utf8, false),
                LanceField::new("chunkId", LanceDataType::Utf8, false),
                LanceField::new("semanticType", LanceDataType::Utf8, false),
            ])),
            vec![
                Arc::new(StringArray::from(vec![format!(
                    "code-ast:{repo_id}:{path}"
                )])),
                Arc::new(StringArray::from(vec!["chunk:0"])),
                Arc::new(StringArray::from(vec!["declaration"])),
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(
            serde_json::json!({
                "repoId": repo_id,
                "path": path,
                "language": "julia",
                "nodeCount": 1,
                "edgeCount": 0,
                "nodes": [],
                "edges": [],
                "projections": [],
                "focusNodeId": line_hint.map(|line| format!("line:{line}")),
                "diagnostics": [],
            })
            .to_string()
            .into_bytes(),
        ))
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingRepoOverviewProvider {
    request: Mutex<Option<RepoOverviewRequestRecord>>,
    call_count: Mutex<usize>,
}

impl RecordingRepoOverviewProvider {
    pub(super) fn recorded_request(&self) -> Option<RepoOverviewRequestRecord> {
        lock_or_panic(&self.request, "repo overview provider record should lock").clone()
    }
}

#[async_trait]
impl RepoOverviewFlightRouteProvider for RecordingRepoOverviewProvider {
    async fn repo_overview_batch(
        &self,
        repo_id: &str,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        *lock_or_panic(&self.request, "repo overview provider record should lock") =
            Some(repo_id.to_string());
        *lock_or_panic(
            &self.call_count,
            "repo overview provider call count should lock",
        ) += 1;
        let batch = LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("repoId", LanceDataType::Utf8, false),
                LanceField::new("displayName", LanceDataType::Utf8, false),
                LanceField::new("revision", LanceDataType::Utf8, true),
                LanceField::new("moduleCount", LanceDataType::Int32, false),
                LanceField::new("symbolCount", LanceDataType::Int32, false),
                LanceField::new("exampleCount", LanceDataType::Int32, false),
                LanceField::new("docCount", LanceDataType::Int32, false),
                LanceField::new("hierarchicalUri", LanceDataType::Utf8, true),
            ])),
            vec![
                Arc::new(StringArray::from(vec![repo_id.to_string()])),
                Arc::new(StringArray::from(vec!["Gateway Sync".to_string()])),
                Arc::new(StringArray::from(vec![Some("rev:123".to_string())])),
                Arc::new(LanceInt32Array::from(vec![3])),
                Arc::new(LanceInt32Array::from(vec![8])),
                Arc::new(LanceInt32Array::from(vec![2])),
                Arc::new(LanceInt32Array::from(vec![5])),
                Arc::new(StringArray::from(vec![Some(format!("repo://{repo_id}"))])),
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(
            serde_json::json!({
                "repoId": repo_id,
                "displayName": "Gateway Sync",
                "revision": "rev:123",
                "moduleCount": 3,
                "symbolCount": 8,
                "exampleCount": 2,
                "docCount": 5,
                "hierarchicalUri": format!("repo://{repo_id}"),
                "hierarchy": ["repo", repo_id],
            })
            .to_string()
            .into_bytes(),
        ))
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingRepoIndexStatusProvider {
    request: Mutex<Option<RepoIndexStatusRequestRecord>>,
    call_count: Mutex<usize>,
}

impl RecordingRepoIndexStatusProvider {
    pub(super) fn recorded_request(&self) -> Option<RepoIndexStatusRequestRecord> {
        lock_or_panic(
            &self.request,
            "repo index status provider record should lock",
        )
        .clone()
    }
}

#[async_trait]
impl RepoIndexStatusFlightRouteProvider for RecordingRepoIndexStatusProvider {
    async fn repo_index_status_batch(
        &self,
        repo_id: Option<&str>,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        *lock_or_panic(
            &self.request,
            "repo index status provider record should lock",
        ) = Some(repo_id.map(ToString::to_string));
        *lock_or_panic(
            &self.call_count,
            "repo index status provider call count should lock",
        ) += 1;
        let batch = LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("total", LanceDataType::Int32, false),
                LanceField::new("queued", LanceDataType::Int32, false),
                LanceField::new("checking", LanceDataType::Int32, false),
                LanceField::new("syncing", LanceDataType::Int32, false),
                LanceField::new("indexing", LanceDataType::Int32, false),
                LanceField::new("ready", LanceDataType::Int32, false),
                LanceField::new("unsupported", LanceDataType::Int32, false),
                LanceField::new("failed", LanceDataType::Int32, false),
                LanceField::new("targetConcurrency", LanceDataType::Int32, false),
                LanceField::new("maxConcurrency", LanceDataType::Int32, false),
                LanceField::new("syncConcurrencyLimit", LanceDataType::Int32, false),
                LanceField::new("currentRepoId", LanceDataType::Utf8, true),
                LanceField::new("reposJson", LanceDataType::Utf8, false),
            ])),
            vec![
                Arc::new(LanceInt32Array::from(vec![3])),
                Arc::new(LanceInt32Array::from(vec![1])),
                Arc::new(LanceInt32Array::from(vec![0])),
                Arc::new(LanceInt32Array::from(vec![1])),
                Arc::new(LanceInt32Array::from(vec![1])),
                Arc::new(LanceInt32Array::from(vec![1])),
                Arc::new(LanceInt32Array::from(vec![0])),
                Arc::new(LanceInt32Array::from(vec![0])),
                Arc::new(LanceInt32Array::from(vec![2])),
                Arc::new(LanceInt32Array::from(vec![4])),
                Arc::new(LanceInt32Array::from(vec![1])),
                Arc::new(StringArray::from(vec![repo_id.map(ToString::to_string)])),
                Arc::new(StringArray::from(vec![
                    serde_json::json!([
                        {
                            "repoId": repo_id.unwrap_or("gateway-sync"),
                            "phase": "ready",
                            "lastRevision": "rev:123",
                            "attemptCount": 2,
                        },
                        {
                            "repoId": "kernel",
                            "phase": "queued",
                            "queuePosition": 1,
                            "attemptCount": 0,
                        }
                    ])
                    .to_string(),
                ])),
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(
            serde_json::json!({
                "total": 3,
                "queued": 1,
                "checking": 0,
                "syncing": 1,
                "indexing": 1,
                "ready": 1,
                "unsupported": 0,
                "failed": 0,
                "targetConcurrency": 2,
                "maxConcurrency": 4,
                "syncConcurrencyLimit": 1,
                "currentRepoId": repo_id,
            })
            .to_string()
            .into_bytes(),
        ))
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingRepoSyncProvider {
    request: Mutex<Option<RepoSyncRequestRecord>>,
    call_count: Mutex<usize>,
}

impl RecordingRepoSyncProvider {
    pub(super) fn recorded_request(&self) -> Option<RepoSyncRequestRecord> {
        lock_or_panic(&self.request, "repo sync provider record should lock").clone()
    }
}

#[async_trait]
impl RepoSyncFlightRouteProvider for RecordingRepoSyncProvider {
    async fn repo_sync_batch(
        &self,
        repo_id: &str,
        mode: &str,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        *lock_or_panic(&self.request, "repo sync provider record should lock") =
            Some((repo_id.to_string(), mode.to_string()));
        *lock_or_panic(
            &self.call_count,
            "repo sync provider call count should lock",
        ) += 1;
        let batch = LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("repoId", LanceDataType::Utf8, false),
                LanceField::new("mode", LanceDataType::Utf8, false),
                LanceField::new("sourceKind", LanceDataType::Utf8, false),
                LanceField::new("refresh", LanceDataType::Utf8, false),
                LanceField::new("mirrorState", LanceDataType::Utf8, false),
                LanceField::new("checkoutState", LanceDataType::Utf8, false),
                LanceField::new("revision", LanceDataType::Utf8, true),
                LanceField::new("checkoutPath", LanceDataType::Utf8, false),
                LanceField::new("mirrorPath", LanceDataType::Utf8, true),
                LanceField::new("checkedAt", LanceDataType::Utf8, false),
                LanceField::new("lastFetchedAt", LanceDataType::Utf8, true),
                LanceField::new("upstreamUrl", LanceDataType::Utf8, true),
                LanceField::new("healthState", LanceDataType::Utf8, false),
                LanceField::new("stalenessState", LanceDataType::Utf8, false),
                LanceField::new("driftState", LanceDataType::Utf8, false),
                LanceField::new("statusSummaryJson", LanceDataType::Utf8, false),
            ])),
            vec![
                Arc::new(StringArray::from(vec![repo_id.to_string()])),
                Arc::new(StringArray::from(vec![mode.to_string()])),
                Arc::new(StringArray::from(vec!["managed_remote".to_string()])),
                Arc::new(StringArray::from(vec!["auto".to_string()])),
                Arc::new(StringArray::from(vec!["validated".to_string()])),
                Arc::new(StringArray::from(vec!["reused".to_string()])),
                Arc::new(StringArray::from(vec![Some("rev:123".to_string())])),
                Arc::new(StringArray::from(vec![format!("/tmp/{repo_id}")])),
                Arc::new(StringArray::from(vec![Some(format!(
                    "/tmp/{repo_id}.mirror"
                ))])),
                Arc::new(StringArray::from(vec!["2026-04-03T19:15:00Z".to_string()])),
                Arc::new(StringArray::from(vec![Some(
                    "2026-04-03T19:10:00Z".to_string(),
                )])),
                Arc::new(StringArray::from(vec![Some(
                    "https://example.com/repo.git".to_string(),
                )])),
                Arc::new(StringArray::from(vec!["healthy".to_string()])),
                Arc::new(StringArray::from(vec!["fresh".to_string()])),
                Arc::new(StringArray::from(vec!["in_sync".to_string()])),
                Arc::new(StringArray::from(vec![
                    serde_json::json!({
                        "healthState": "healthy",
                        "driftState": "in_sync",
                        "attentionRequired": false,
                    })
                    .to_string(),
                ])),
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(
            serde_json::json!({
                "repoId": repo_id,
                "mode": mode,
                "sourceKind": "managed_remote",
                "refresh": "auto",
                "mirrorState": "validated",
                "checkoutState": "reused",
                "revision": "rev:123",
                "checkoutPath": format!("/tmp/{repo_id}"),
                "mirrorPath": format!("/tmp/{repo_id}.mirror"),
                "checkedAt": "2026-04-03T19:15:00Z",
                "lastFetchedAt": "2026-04-03T19:10:00Z",
                "upstreamUrl": "https://example.com/repo.git",
                "healthState": "healthy",
                "stalenessState": "fresh",
                "driftState": "in_sync",
                "statusSummary": {
                    "healthState": "healthy",
                    "driftState": "in_sync",
                    "attentionRequired": false,
                },
            })
            .to_string()
            .into_bytes(),
        ))
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingRepoDocCoverageProvider {
    request: Mutex<Option<RepoDocCoverageRequestRecord>>,
    call_count: Mutex<usize>,
}

impl RecordingRepoDocCoverageProvider {
    pub(super) fn recorded_request(&self) -> Option<RepoDocCoverageRequestRecord> {
        lock_or_panic(
            &self.request,
            "repo doc coverage provider record should lock",
        )
        .clone()
    }
}

#[async_trait]
impl RepoDocCoverageFlightRouteProvider for RecordingRepoDocCoverageProvider {
    async fn repo_doc_coverage_batch(
        &self,
        repo_id: &str,
        module_id: Option<&str>,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        *lock_or_panic(
            &self.request,
            "repo doc coverage provider record should lock",
        ) = Some((repo_id.to_string(), module_id.map(ToString::to_string)));
        *lock_or_panic(
            &self.call_count,
            "repo doc coverage provider call count should lock",
        ) += 1;
        let batch = LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("repoId", LanceDataType::Utf8, false),
                LanceField::new("docId", LanceDataType::Utf8, false),
                LanceField::new("title", LanceDataType::Utf8, false),
                LanceField::new("path", LanceDataType::Utf8, false),
                LanceField::new("format", LanceDataType::Utf8, true),
            ])),
            vec![
                Arc::new(StringArray::from(vec![repo_id.to_string()])),
                Arc::new(StringArray::from(vec![format!("doc:{repo_id}")])),
                Arc::new(StringArray::from(vec!["Repo Doc".to_string()])),
                Arc::new(StringArray::from(vec!["docs/index.md".to_string()])),
                Arc::new(StringArray::from(vec![Some("markdown".to_string())])),
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(
            serde_json::json!({
                "repoId": repo_id,
                "moduleId": module_id,
                "coveredSymbols": 3,
                "uncoveredSymbols": 1,
                "hierarchicalUri": format!("repo://{repo_id}/docs"),
                "hierarchy": ["repo", repo_id],
            })
            .to_string()
            .into_bytes(),
        ))
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingDocumentExtractProvider {
    request: Mutex<Option<DocumentExtractRequestRecord>>,
    call_count: Mutex<usize>,
    status_call_count: Mutex<usize>,
}

impl RecordingDocumentExtractProvider {
    pub(super) fn recorded_request(&self) -> Option<DocumentExtractRequestRecord> {
        lock_or_panic(
            &self.request,
            "document extract provider record should lock",
        )
        .clone()
    }

    pub(super) fn call_count(&self) -> usize {
        *lock_or_panic(
            &self.call_count,
            "document extract provider call count should lock",
        )
    }

    pub(super) fn status_call_count(&self) -> usize {
        *lock_or_panic(
            &self.status_call_count,
            "document extract status provider call count should lock",
        )
    }
}

#[async_trait]
impl DocumentExtractFlightRouteProvider for RecordingDocumentExtractProvider {
    async fn document_extract_batch(
        &self,
        source_path: &str,
        output_dir: &str,
        force: bool,
        error_row: bool,
        profile: &str,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        *lock_or_panic(
            &self.request,
            "document extract provider record should lock",
        ) = Some((
            source_path.to_string(),
            output_dir.to_string(),
            force,
            error_row,
            profile.to_string(),
        ));
        *lock_or_panic(
            &self.call_count,
            "document extract provider call count should lock",
        ) += 1;
        let batch = ArrowRecordBatch::try_new(
            Arc::new(ArrowSchema::new(vec![
                ArrowField::new("sourcePath", ArrowDataType::Utf8, false),
                ArrowField::new("resourceType", ArrowDataType::Utf8, false),
                ArrowField::new("resourcePath", ArrowDataType::Utf8, false),
                ArrowField::new("pageIndex", ArrowDataType::Int32, false),
                ArrowField::new("status", ArrowDataType::Utf8, false),
            ])),
            vec![
                Arc::new(ArrowStringArray::from(vec![source_path.to_string()])),
                Arc::new(ArrowStringArray::from(vec!["document".to_string()])),
                Arc::new(ArrowStringArray::from(vec![format!(
                    "{source_path}.extracted/_main.md"
                )])),
                Arc::new(ArrowInt32Array::from(vec![0])),
                Arc::new(ArrowStringArray::from(vec!["ok".to_string()])),
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(DocumentExtractFlightRouteResponse::new(batch))
    }

    async fn document_extract_status_batch(
        &self,
        job_id: &str,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let mut call_count = lock_or_panic(
            &self.status_call_count,
            "document extract status provider call count should lock",
        );
        *call_count += 1;
        let status = if *call_count == 1 {
            "queued"
        } else {
            "running"
        };
        let attempt_count = i32::try_from(*call_count).unwrap_or(i32::MAX);
        let batch = ArrowRecordBatch::try_new(
            Arc::new(ArrowSchema::new(vec![
                ArrowField::new("jobId", ArrowDataType::Utf8, false),
                ArrowField::new("sourcePath", ArrowDataType::Utf8, false),
                ArrowField::new("outputDir", ArrowDataType::Utf8, false),
                ArrowField::new("contentHash", ArrowDataType::Utf8, false),
                ArrowField::new("status", ArrowDataType::Utf8, false),
                ArrowField::new("attemptCount", ArrowDataType::Int32, false),
                ArrowField::new("createdAtMs", ArrowDataType::Int64, false),
                ArrowField::new("startedAtMs", ArrowDataType::Int64, false),
                ArrowField::new("finishedAtMs", ArrowDataType::Int64, false),
                ArrowField::new("errorMessage", ArrowDataType::Utf8, false),
            ])),
            vec![
                Arc::new(ArrowStringArray::from(vec![job_id.to_string()])),
                Arc::new(ArrowStringArray::from(vec!["docs/manual.pdf".to_string()])),
                Arc::new(ArrowStringArray::from(vec![
                    ".cache/document-extract".to_string(),
                ])),
                Arc::new(ArrowStringArray::from(vec!["hash:manual".to_string()])),
                Arc::new(ArrowStringArray::from(vec![status.to_string()])),
                Arc::new(ArrowInt32Array::from(vec![attempt_count])),
                Arc::new(ArrowInt64Array::from(vec![100_i64])),
                Arc::new(ArrowInt64Array::from(vec![200_i64])),
                Arc::new(ArrowInt64Array::from(vec![0_i64])),
                Arc::new(ArrowStringArray::from(vec![String::new()])),
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(DocumentExtractFlightRouteResponse::new(batch))
    }
}
