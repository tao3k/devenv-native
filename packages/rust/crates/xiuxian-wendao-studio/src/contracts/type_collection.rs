//! `Specta` type collection assembly for lightweight Studio contracts.

use specta::TypeCollection;

use super::{
    ApiError, AstSearchResponse, AttachmentSearchResponse, AutocompleteResponse,
    CodeAstAnalysisResponse, DefinitionResolveResponse, DocumentExtractJobStatus,
    DocumentExtractJobSubmitRequest, DocumentExtractJobsStatus, DocumentExtractResource,
    DocumentExtractResult, GraphNeighborsResponse, MarkdownAnalysisResponse,
    ReferenceSearchResponse, SearchResponse, SymbolSearchResponse, Topology3dPayload,
    UiCapabilities, UiConfig, UiPluginArtifact, UiPluginLaunchSpec, VfsContentResponse, VfsEntry,
    VfsScanEntry, VfsScanResult,
};

/// Build the lightweight Studio Specta type collection.
#[must_use]
pub fn studio_type_collection() -> TypeCollection {
    TypeCollection::default()
        .register::<ApiError>()
        .register::<VfsEntry>()
        .register::<VfsScanEntry>()
        .register::<VfsScanResult>()
        .register::<VfsContentResponse>()
        .register::<DocumentExtractResult>()
        .register::<DocumentExtractJobSubmitRequest>()
        .register::<DocumentExtractJobStatus>()
        .register::<DocumentExtractJobsStatus>()
        .register::<UiPluginArtifact>()
        .register::<UiPluginLaunchSpec>()
        .register::<DocumentExtractResource>()
}

/// Build the frontend-facing Studio Specta type collection.
#[must_use]
pub fn studio_frontend_type_collection() -> TypeCollection {
    TypeCollection::default()
        .register::<ApiError>()
        .register::<VfsEntry>()
        .register::<VfsScanEntry>()
        .register::<VfsScanResult>()
        .register::<VfsContentResponse>()
        .register::<UiCapabilities>()
        .register::<UiConfig>()
        .register::<GraphNeighborsResponse>()
        .register::<Topology3dPayload>()
        .register::<SearchResponse>()
        .register::<AttachmentSearchResponse>()
        .register::<AstSearchResponse>()
        .register::<DefinitionResolveResponse>()
        .register::<ReferenceSearchResponse>()
        .register::<SymbolSearchResponse>()
        .register::<AutocompleteResponse>()
        .register::<MarkdownAnalysisResponse>()
        .register::<CodeAstAnalysisResponse>()
        .register::<DocumentExtractResult>()
        .register::<DocumentExtractJobSubmitRequest>()
        .register::<DocumentExtractJobStatus>()
        .register::<DocumentExtractJobsStatus>()
}
