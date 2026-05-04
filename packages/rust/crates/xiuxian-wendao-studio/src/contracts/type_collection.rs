use specta::TypeCollection;

use super::{
    ApiError, DocumentExtractJobStatus, DocumentExtractJobSubmitRequest, DocumentExtractJobsStatus,
    DocumentExtractResource, DocumentExtractResult, UiPluginArtifact, UiPluginLaunchSpec,
    VfsContentResponse, VfsEntry, VfsScanEntry, VfsScanResult,
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
