use std::path::Path;

use super::support::{collect_rust_source_occurrences, workspace_root};

const STUDIO_TYPE_COLLECTION_SYMBOLS: &[&str] =
    &["studio_type_collection", "studio_frontend_type_collection"];
const STUDIO_PLUGIN_ARTIFACT_SYMBOLS: &[&str] = &[
    "UiPluginArtifact",
    "UiPluginLaunchSpec",
    "UiPluginTransportKind",
];
const STUDIO_HTTP_API_SYMBOLS: &[&str] = &[
    "ApiError",
    "VfsEntry",
    "VfsCategory",
    "VfsScanEntry",
    "VfsScanResult",
    "VfsContentResponse",
    "DocumentExtractResult",
    "DocumentExtractResource",
    "DocumentExtractJobSubmitRequest",
    "DocumentExtractJobStatus",
    "DocumentExtractJobsStatus",
];
const STUDIO_GRAPH_API_SYMBOLS: &[&str] = &[
    "GraphNode",
    "GraphLink",
    "GraphNeighborsResponse",
    "Topology3dPayload",
    "TopologyNode",
    "TopologyLink",
    "TopologyCluster",
];
const STUDIO_SYMBOL_API_SYMBOLS: &[&str] = &[
    "SymbolSearchHit",
    "SymbolSearchResponse",
    "AutocompleteHit",
    "AutocompleteResponse",
];
const STUDIO_SEARCH_RESPONSE_SYMBOLS: &[&str] = &[
    "AttachmentSearchResponse",
    "DefinitionResolveResponse",
    "ReferenceSearchResponse",
    "SearchResponse",
];
const STUDIO_MARKDOWN_ANALYSIS_API_SYMBOLS: &[&str] = &[
    "AnalysisEdge",
    "AnalysisEdgeKind",
    "AnalysisEvidence",
    "MarkdownRetrievalAtom",
    "MarkdownAnalysisDocumentLink",
    "MarkdownAnalysisDocumentLinkKind",
    "MarkdownAnalysisDocumentMetadata",
    "MarkdownAnalysisResponse",
    "MermaidProjection",
    "MermaidViewKind",
];
const STUDIO_SEARCH_MANIFEST_SYMBOLS: &[&str] = &[
    "UiConfig",
    "UiProjectConfig",
    "UiRepoProjectConfig",
    "UiCapabilities",
    "UiSearchContract",
    "UiCodeSearchContract",
    "UiCodeSearchContractExample",
    "UiCodeSearchRoutes",
    "UiSearchContractAlias",
    "UiRepoDiscoveryContract",
    "UiRepoDiscoverySurfaceContract",
];

#[test]
fn wendao_domain_contracts_do_not_export_studio_search_manifest_dtos() {
    assert_domain_contracts_do_not_contain_symbols(
        STUDIO_SEARCH_MANIFEST_SYMBOLS,
        "Studio capability and search-manifest DTOs belong to xiuxian-wendao-studio contracts, not xiuxian-wendao search contracts",
    );
}

#[test]
fn wendao_domain_contracts_do_not_export_studio_type_collections() {
    assert_domain_contracts_do_not_contain_symbols(
        STUDIO_TYPE_COLLECTION_SYMBOLS,
        "Studio TypeScript schema collection helpers belong to xiuxian-wendao-studio contracts, not xiuxian-wendao search contracts",
    );
}

#[test]
fn wendao_domain_contracts_do_not_export_studio_plugin_artifact_dtos() {
    assert_domain_contracts_do_not_contain_symbols(
        STUDIO_PLUGIN_ARTIFACT_SYMBOLS,
        "Studio plugin artifact DTOs belong to xiuxian-wendao-studio contracts, not xiuxian-wendao search contracts",
    );
}

#[test]
fn wendao_domain_contracts_do_not_export_studio_http_api_dtos() {
    assert_domain_contracts_do_not_contain_symbols(
        STUDIO_HTTP_API_SYMBOLS,
        "Studio HTTP API DTOs belong to xiuxian-wendao-studio contracts, not xiuxian-wendao search contracts",
    );
}

#[test]
fn wendao_domain_contracts_do_not_export_studio_graph_api_dtos() {
    assert_domain_contracts_do_not_contain_symbols(
        STUDIO_GRAPH_API_SYMBOLS,
        "Studio graph/topology API DTOs belong to xiuxian-wendao-studio contracts, not xiuxian-wendao search contracts",
    );
}

#[test]
fn wendao_domain_contracts_do_not_export_studio_symbol_api_dtos() {
    assert_domain_contracts_do_not_contain_symbols(
        STUDIO_SYMBOL_API_SYMBOLS,
        "Studio symbol and autocomplete API wrapper DTOs belong to xiuxian-wendao-studio contracts, not xiuxian-wendao search contracts",
    );
}

#[test]
fn wendao_domain_contracts_do_not_export_studio_search_response_dtos() {
    assert_domain_contracts_do_not_contain_symbols(
        STUDIO_SEARCH_RESPONSE_SYMBOLS,
        "Studio search response wrapper DTOs belong to xiuxian-wendao-studio contracts, not xiuxian-wendao search contracts",
    );
}

#[test]
fn wendao_domain_contracts_do_not_export_studio_markdown_analysis_dtos() {
    assert_domain_contracts_do_not_contain_symbols(
        STUDIO_MARKDOWN_ANALYSIS_API_SYMBOLS,
        "Studio Markdown analysis edge, projection, and response DTOs belong to xiuxian-wendao-studio contracts, not xiuxian-wendao search contracts",
    );
}

fn assert_domain_contracts_do_not_contain_symbols(symbols: &[&str], message: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let domain_contracts_root = workspace_root(manifest_dir)
        .join("packages/rust/crates/xiuxian-wendao/src/search/contracts");
    let mut offenders = Vec::new();

    for symbol in symbols {
        collect_rust_source_occurrences(domain_contracts_root.as_path(), symbol, &mut offenders);
    }

    assert!(offenders.is_empty(), "{message}:\n{}", offenders.join("\n"));
}
