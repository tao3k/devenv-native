use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::types::DocumentExtractJobStatus;
use crate::gateway::studio::router::handlers::analysis::document_extract::arrow_cache::DOCUMENT_RESOURCE_ARROW_CACHE_NAME;

impl DocumentExtractJobStatus {
    pub(super) fn with_output_dir(&self, output_dir: &Path) -> Self {
        Self {
            output_dir: output_dir.to_string_lossy().to_string(),
            ..self.clone()
        }
    }
}

pub(crate) fn default_output_dir(source_path: &Path) -> PathBuf {
    let Some(extension) = source_path.extension().and_then(std::ffi::OsStr::to_str) else {
        return source_path.with_extension("extracted");
    };
    source_path.with_extension(format!("{extension}.extracted"))
}

pub(in super::super) fn artifact_ready(status: &DocumentExtractJobStatus) -> bool {
    Path::new(status.artifact_dir.as_str())
        .join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME)
        .exists()
        && Path::new(status.artifact_dir.as_str())
            .join("_complete.marker")
            .exists()
}

pub(super) fn now_ms() -> i64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}
