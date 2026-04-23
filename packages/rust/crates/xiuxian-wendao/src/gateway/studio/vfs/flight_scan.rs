use std::sync::Arc;

use async_trait::async_trait;
use tonic::Status;
use xiuxian_db_store::{
    LanceBooleanArray, LanceDataType, LanceField, LanceRecordBatch, LanceSchema, LanceStringArray,
    LanceUInt64Array,
};
use xiuxian_wendao_runtime::transport::{VfsScanFlightRouteProvider, VfsScanFlightRouteResponse};

use crate::gateway::studio::StudioState;
use crate::gateway::studio::types::{VfsCategory, VfsScanEntry, VfsScanResult};

use super::scan::scan_roots;

/// Studio-backed Flight provider for the semantic `/vfs/scan` route.
#[derive(Clone)]
pub(crate) struct StudioVfsScanFlightRouteProvider {
    studio: Arc<StudioState>,
}

impl StudioVfsScanFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(studio: Arc<StudioState>) -> Self {
        Self { studio }
    }
}

impl std::fmt::Debug for StudioVfsScanFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioVfsScanFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl VfsScanFlightRouteProvider for StudioVfsScanFlightRouteProvider {
    async fn scan_vfs_batch(&self) -> Result<VfsScanFlightRouteResponse, Status> {
        load_vfs_scan_flight_response(self.studio.as_ref()).map_err(Status::internal)
    }
}

pub(crate) fn load_vfs_scan_flight_response(
    studio: &StudioState,
) -> Result<VfsScanFlightRouteResponse, String> {
    let response = scan_roots(studio);
    let batch = vfs_scan_result_batch(&response)?;
    let app_metadata = vfs_scan_result_flight_app_metadata(&response)?;
    Ok(VfsScanFlightRouteResponse::new(batch).with_app_metadata(app_metadata))
}

pub(crate) fn vfs_scan_result_batch(response: &VfsScanResult) -> Result<LanceRecordBatch, String> {
    let entries = response.entries.as_slice();
    let project_dirs_json = project_dirs_json_values(entries)?;
    LanceRecordBatch::try_new(
        vfs_scan_batch_schema(),
        vec![
            required_utf8_column(entries, |entry| entry.path.as_str()),
            required_utf8_column(entries, |entry| entry.name.as_str()),
            boolean_column(entries, |entry| entry.is_dir),
            required_utf8_column(entries, |entry| vfs_category_as_str(entry.category)),
            u64_column(entries, |entry| entry.size),
            u64_column(entries, |entry| entry.modified),
            optional_utf8_column(entries, |entry| entry.content_type.as_deref()),
            boolean_column(entries, |entry| entry.has_frontmatter),
            optional_utf8_column(entries, |entry| entry.wendao_id.as_deref()),
            optional_utf8_column(entries, |entry| entry.project_name.as_deref()),
            optional_utf8_column(entries, |entry| entry.root_label.as_deref()),
            optional_utf8_column(entries, |entry| entry.project_root.as_deref()),
            optional_string_column(project_dirs_json.as_slice()),
        ],
    )
    .map_err(|error| format!("failed to build VFS scan Flight batch: {error}"))
}

pub(crate) fn vfs_scan_result_flight_app_metadata(
    response: &VfsScanResult,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&serde_json::json!({
        "fileCount": response.file_count,
        "dirCount": response.dir_count,
        "scanDurationMs": response.scan_duration_ms,
    }))
    .map_err(|error| format!("failed to encode VFS scan Flight app metadata: {error}"))
}

fn encode_project_dirs_json(project_dirs: &[String]) -> Result<String, String> {
    serde_json::to_string(project_dirs)
        .map_err(|error| format!("failed to encode VFS scan project dirs: {error}"))
}

fn project_dirs_json_values(entries: &[VfsScanEntry]) -> Result<Vec<Option<String>>, String> {
    entries
        .iter()
        .map(|entry| {
            entry
                .project_dirs
                .as_ref()
                .map(|project_dirs| encode_project_dirs_json(project_dirs.as_slice()))
                .transpose()
        })
        .collect()
}

fn vfs_scan_batch_schema() -> Arc<LanceSchema> {
    Arc::new(LanceSchema::new(vec![
        LanceField::new("path", LanceDataType::Utf8, false),
        LanceField::new("name", LanceDataType::Utf8, false),
        LanceField::new("isDir", LanceDataType::Boolean, false),
        LanceField::new("category", LanceDataType::Utf8, false),
        LanceField::new("size", LanceDataType::UInt64, false),
        LanceField::new("modified", LanceDataType::UInt64, false),
        LanceField::new("contentType", LanceDataType::Utf8, true),
        LanceField::new("hasFrontmatter", LanceDataType::Boolean, false),
        LanceField::new("wendaoId", LanceDataType::Utf8, true),
        LanceField::new("projectName", LanceDataType::Utf8, true),
        LanceField::new("rootLabel", LanceDataType::Utf8, true),
        LanceField::new("projectRoot", LanceDataType::Utf8, true),
        LanceField::new("projectDirsJson", LanceDataType::Utf8, true),
    ]))
}

fn required_utf8_column<'a>(
    entries: &'a [VfsScanEntry],
    select: impl Fn(&'a VfsScanEntry) -> &'a str,
) -> Arc<LanceStringArray> {
    Arc::new(LanceStringArray::from(
        entries.iter().map(select).collect::<Vec<_>>(),
    ))
}

fn optional_utf8_column<'a>(
    entries: &'a [VfsScanEntry],
    select: impl Fn(&'a VfsScanEntry) -> Option<&'a str>,
) -> Arc<LanceStringArray> {
    Arc::new(LanceStringArray::from(
        entries.iter().map(select).collect::<Vec<_>>(),
    ))
}

fn optional_string_column(values: &[Option<String>]) -> Arc<LanceStringArray> {
    Arc::new(LanceStringArray::from(
        values
            .iter()
            .map(|value| value.as_deref())
            .collect::<Vec<_>>(),
    ))
}

fn boolean_column(
    entries: &[VfsScanEntry],
    select: impl Fn(&VfsScanEntry) -> bool,
) -> Arc<LanceBooleanArray> {
    Arc::new(LanceBooleanArray::from(
        entries.iter().map(select).collect::<Vec<_>>(),
    ))
}

fn u64_column(
    entries: &[VfsScanEntry],
    select: impl Fn(&VfsScanEntry) -> u64,
) -> Arc<LanceUInt64Array> {
    Arc::new(LanceUInt64Array::from(
        entries.iter().map(select).collect::<Vec<_>>(),
    ))
}

fn vfs_category_as_str(category: VfsCategory) -> &'static str {
    match category {
        VfsCategory::Folder => "folder",
        VfsCategory::Skill => "skill",
        VfsCategory::Doc => "doc",
        VfsCategory::Knowledge => "knowledge",
        VfsCategory::Other => "other",
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/gateway/studio/vfs/flight_scan.rs"]
mod tests;
