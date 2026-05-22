//! Activity output artifact writing helpers.

use std::fs;
use std::io;
use std::path::Path;

use sha2::{Digest, Sha256};
use xiuxian_qianji_control::{ArtifactId, ArtifactKind, ArtifactRef, WorkerActivityTask};

use crate::qianji_cli::invalid_input;

const DEFAULT_OUTPUT_ARTIFACT_KIND: &str = "activity.output";

#[derive(Clone, Copy)]
pub(crate) struct ActivityOutputArtifactRequest<'a> {
    pub(crate) path: &'a Path,
    pub(crate) content: &'a str,
    pub(crate) artifact_id: Option<&'a str>,
    pub(crate) artifact_kind: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ActivityOutputArtifact {
    pub(crate) output_ref: ArtifactRef,
    pub(crate) output_hash: String,
}

pub(crate) fn write_activity_output_artifact(
    task: &WorkerActivityTask,
    request: ActivityOutputArtifactRequest<'_>,
) -> io::Result<ActivityOutputArtifact> {
    let content = request.content.as_bytes();
    write_new_or_matching_file(request.path, content)?;
    let output_hash = sha256_digest(content);
    let default_artifact_id = default_output_artifact_id(task);
    let artifact_id = ArtifactId::new(request.artifact_id.unwrap_or(default_artifact_id.as_str()))
        .map_err(|error| invalid_input(format!("{error}")))?;
    let artifact_kind = ArtifactKind::new(
        request
            .artifact_kind
            .unwrap_or(DEFAULT_OUTPUT_ARTIFACT_KIND),
    )
    .map_err(|error| invalid_input(format!("{error}")))?;
    Ok(ActivityOutputArtifact {
        output_ref: ArtifactRef {
            artifact_id,
            artifact_kind,
            uri: request.path.display().to_string(),
            content_digest: Some(output_hash.clone()),
            metadata: serde_json::json!({
                "source": "qianji-control-activity-worker-once",
                "path": request.path.display().to_string(),
            }),
        },
        output_hash,
    })
}

fn write_new_or_matching_file(path: &Path, content: &[u8]) -> io::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| {
            io::Error::other(format!(
                "failed to create activity output artifact directory `{}`: {error}",
                parent.display()
            ))
        })?;
    }
    if path.exists() {
        let existing = fs::read(path).map_err(|error| {
            io::Error::other(format!(
                "failed to read existing activity output artifact `{}`: {error}",
                path.display()
            ))
        })?;
        if existing != content {
            return Err(invalid_input(format!(
                "activity output artifact `{}` already exists with different content",
                path.display()
            )));
        }
        return Ok(());
    }
    fs::write(path, content).map_err(|error| {
        io::Error::other(format!(
            "failed to write activity output artifact `{}`: {error}",
            path.display()
        ))
    })
}

fn default_output_artifact_id(task: &WorkerActivityTask) -> String {
    format!("{}-output", task.activity_id.as_str())
}

fn sha256_digest(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let digest = hasher.finalize();
    format!("sha256:{digest:x}")
}
