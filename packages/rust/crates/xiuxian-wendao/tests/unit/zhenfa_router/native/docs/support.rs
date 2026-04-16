pub(super) use std::sync::Arc;

pub(super) use serde_json::json;
pub(super) use xiuxian_zhenfa::{ZhenfaContext, ZhenfaRegistry, ZhenfaTool};

pub(super) use crate::analyzers::{
    DocsDocumentSegmentResult, DocsToolRuntime, DocsToolRuntimeHandle,
};
pub(super) use crate::analyzers::{
    DocsNavigationOptions, DocsPageIndexDocumentsResult, DocsPageIndexNodeResult,
    DocsPageIndexTreeResult, DocsPageIndexTreeSearchResult, DocsPageIndexTreesResult,
    DocsPageResult, DocsRetrievalContextOptions, DocsRetrievalContextResult, DocsToolService,
    ProjectedPageIndexDocument, ProjectedPageIndexNodeHit, ProjectedPageRecord, ProjectionPageKind,
    RepoIntelligenceError,
};
pub(super) use crate::zhenfa_router::native::{
    WendaoContextExt, WendaoDocsGetDocumentNodeTool, WendaoDocsGetDocumentSegmentTool,
    WendaoDocsGetDocumentStructureCatalogTool, WendaoDocsGetDocumentStructureOutlineTool,
    WendaoDocsGetDocumentTool, WendaoDocsGetTocDocumentsTool,
    WendaoDocsSearchDocumentStructureTool, register_wendao_docs_native_tools,
    resolve_docs_tool_runtime,
};

pub(super) type TestResult = Result<(), Box<dyn std::error::Error>>;

pub(super) const TEST_REPO_ID: &str = "modelica-docs-native-tool";
pub(super) const TEST_PAGE_ID: &str = "repo:modelica-docs-native-tool:projection:reference:symbol:repo:modelica-docs-native-tool:symbol:Projectionica.Controllers.PI";

#[derive(Default)]
pub(super) struct FakeDocsToolRuntime;

impl DocsToolRuntime for FakeDocsToolRuntime {
    fn get_document(&self, page_id: &str) -> Result<DocsPageResult, RepoIntelligenceError> {
        Ok(DocsPageResult {
            repo_id: TEST_REPO_ID.to_string(),
            page: ProjectedPageRecord {
                repo_id: TEST_REPO_ID.to_string(),
                page_id: page_id.to_string(),
                kind: ProjectionPageKind::Reference,
                title: "Projectionica.Controllers.PI".to_string(),
                module_ids: Vec::new(),
                symbol_ids: vec![
                    "repo:modelica-docs-native-tool:symbol:Projectionica.Controllers.PI"
                        .to_string(),
                ],
                example_ids: Vec::new(),
                doc_ids: Vec::new(),
                paths: vec!["Projectionica/Controllers/PI.mo".to_string()],
                format_hints: vec!["modelica".to_string()],
                sections: Vec::new(),
                doc_id: String::new(),
                path: "Projectionica/Controllers/PI.mo".to_string(),
                keywords: vec![
                    "Projectionica.Controllers.PI".to_string(),
                    "Projectionica/Controllers/PI.mo".to_string(),
                    "modelica".to_string(),
                ],
            },
        })
    }

    fn get_document_structure(
        &self,
        _page_id: &str,
    ) -> Result<DocsPageIndexTreeResult, RepoIntelligenceError> {
        panic!("get_document_structure is not used in this test")
    }

    fn get_document_structure_outline(
        &self,
        _page_id: &str,
    ) -> Result<DocsPageIndexTreeResult, RepoIntelligenceError> {
        Ok(DocsPageIndexTreeResult {
            repo_id: TEST_REPO_ID.to_string(),
            tree: Some(crate::analyzers::ProjectedPageIndexTree {
                repo_id: TEST_REPO_ID.to_string(),
                page_id: TEST_PAGE_ID.to_string(),
                kind: ProjectionPageKind::Reference,
                path: "reference/projectionica-controllers-pi.md".to_string(),
                doc_id: "doc:projectionica-controllers-pi".to_string(),
                title: "Projectionica.Controllers.PI".to_string(),
                root_count: 1,
                roots: vec![crate::analyzers::ProjectedPageIndexNode {
                    node_id: "outline:1".to_string(),
                    title: "Anchors".to_string(),
                    level: 2,
                    structural_path: vec![
                        "Projectionica.Controllers.PI".to_string(),
                        "Anchors".to_string(),
                    ],
                    line_range: (12, 18),
                    token_count: 4,
                    is_thinned: false,
                    text: String::new(),
                    summary: Some("Anchor summary".to_string()),
                    children: Vec::new(),
                }],
            }),
        })
    }

    fn get_document_structure_catalog(
        &self,
    ) -> Result<DocsPageIndexTreesResult, RepoIntelligenceError> {
        Ok(DocsPageIndexTreesResult {
            repo_id: TEST_REPO_ID.to_string(),
            trees: vec![crate::analyzers::ProjectedPageIndexTree {
                repo_id: TEST_REPO_ID.to_string(),
                page_id: TEST_PAGE_ID.to_string(),
                kind: ProjectionPageKind::Reference,
                path: "reference/projectionica-controllers-pi.md".to_string(),
                doc_id: "doc:projectionica-controllers-pi".to_string(),
                title: "Projectionica.Controllers.PI".to_string(),
                root_count: 1,
                roots: vec![crate::analyzers::ProjectedPageIndexNode {
                    node_id: "catalog:1".to_string(),
                    title: "Anchors".to_string(),
                    level: 2,
                    structural_path: vec![
                        "Projectionica.Controllers.PI".to_string(),
                        "Anchors".to_string(),
                    ],
                    line_range: (12, 18),
                    token_count: 4,
                    is_thinned: false,
                    text: String::new(),
                    summary: Some("Anchor summary".to_string()),
                    children: Vec::new(),
                }],
            }],
        })
    }

    fn get_document_segment(
        &self,
        page_id: &str,
        line_start: usize,
        line_end: usize,
    ) -> Result<DocsDocumentSegmentResult, RepoIntelligenceError> {
        Ok(DocsDocumentSegmentResult {
            repo_id: TEST_REPO_ID.to_string(),
            page_id: page_id.to_string(),
            kind: ProjectionPageKind::Reference,
            path: "reference/projectionica-controllers-pi.md".to_string(),
            title: "Projectionica.Controllers.PI".to_string(),
            line_range: (line_start, line_end),
            line_count: 18,
            content: "## Anchors\nBody".to_string(),
        })
    }

    fn get_document_node(
        &self,
        page_id: &str,
        node_id: &str,
    ) -> Result<DocsPageIndexNodeResult, RepoIntelligenceError> {
        Ok(DocsPageIndexNodeResult {
            repo_id: TEST_REPO_ID.to_string(),
            hit: Some(ProjectedPageIndexNodeHit {
                repo_id: TEST_REPO_ID.to_string(),
                page_id: page_id.to_string(),
                page_title: "Projectionica.Controllers.PI".to_string(),
                page_kind: ProjectionPageKind::Reference,
                path: "reference/projectionica-controllers-pi.md".to_string(),
                doc_id: "doc:projectionica-controllers-pi".to_string(),
                node_id: node_id.to_string(),
                node_title: "Anchors".to_string(),
                structural_path: vec![
                    "Projectionica.Controllers.PI".to_string(),
                    "Anchors".to_string(),
                ],
                line_range: (12, 18),
                text: "## Anchors\nBody".to_string(),
            }),
        })
    }

    fn search_document_structure(
        &self,
        query: &str,
        kind: Option<ProjectionPageKind>,
        limit: usize,
    ) -> Result<DocsPageIndexTreeSearchResult, RepoIntelligenceError> {
        Ok(DocsPageIndexTreeSearchResult {
            repo_id: TEST_REPO_ID.to_string(),
            hits: vec![ProjectedPageIndexNodeHit {
                repo_id: TEST_REPO_ID.to_string(),
                page_id: TEST_PAGE_ID.to_string(),
                page_title: "Projectionica.Controllers.PI".to_string(),
                page_kind: kind.unwrap_or(ProjectionPageKind::Reference),
                path: "reference/projectionica-controllers-pi.md".to_string(),
                doc_id: "doc:projectionica-controllers-pi".to_string(),
                node_id: format!("search:{limit}"),
                node_title: query.to_ascii_uppercase(),
                structural_path: vec![
                    "Projectionica.Controllers.PI".to_string(),
                    query.to_string(),
                ],
                line_range: (12, 18),
                text: "## Anchors\nBody".to_string(),
            }],
        })
    }

    fn get_toc_documents(&self) -> Result<DocsPageIndexDocumentsResult, RepoIntelligenceError> {
        Ok(DocsPageIndexDocumentsResult {
            repo_id: TEST_REPO_ID.to_string(),
            documents: vec![ProjectedPageIndexDocument {
                repo_id: TEST_REPO_ID.to_string(),
                page_id: TEST_PAGE_ID.to_string(),
                path: "reference/projectionica-controllers-pi.md".to_string(),
                doc_id: "doc:projectionica-controllers-pi".to_string(),
                title: "Projectionica.Controllers.PI".to_string(),
                sections: Vec::new(),
            }],
        })
    }

    fn get_navigation_with_options(
        &self,
        _page_id: &str,
        _options: DocsNavigationOptions,
    ) -> Result<crate::analyzers::DocsNavigationResult, RepoIntelligenceError> {
        panic!("get_navigation_with_options is not used in this test")
    }

    fn get_retrieval_context_with_options(
        &self,
        _page_id: &str,
        _options: DocsRetrievalContextOptions,
    ) -> Result<DocsRetrievalContextResult, RepoIntelligenceError> {
        panic!("get_retrieval_context_with_options is not used in this test")
    }
}

pub(super) fn current_thread_runtime() -> Result<tokio::runtime::Runtime, std::io::Error> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
}

pub(super) fn docs_context_with_fake_runtime() -> ZhenfaContext {
    let runtime: Arc<dyn DocsToolRuntime> = Arc::new(FakeDocsToolRuntime);
    let mut ctx = ZhenfaContext::default();
    let _ = ctx.insert_extension(DocsToolRuntimeHandle::new(runtime));
    ctx
}
