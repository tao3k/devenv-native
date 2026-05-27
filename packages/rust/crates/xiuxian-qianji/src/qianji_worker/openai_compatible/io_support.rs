use std::fs;
use std::io;
use std::path::PathBuf;

use xiuxian_qianji_control::ArtifactRef;

use crate::qianji_worker::invalid_input;

pub(super) fn read_local_artifact_text(artifact_ref: &ArtifactRef) -> io::Result<String> {
    let uri = artifact_ref.uri.trim();
    if uri.is_empty() {
        return Err(invalid_input("LLM artifact URI must not be blank"));
    }
    if uri.starts_with("artifact://") || uri.starts_with("http://") || uri.starts_with("https://") {
        return Err(invalid_input(format!(
            "OpenAI-compatible executor can only materialize local file artifacts in this slice, got `{uri}`"
        )));
    }
    fs::read_to_string(local_artifact_path(uri)).map_err(|error| {
        io::Error::other(format!(
            "failed to read LLM local artifact `{uri}`: {error}"
        ))
    })
}

fn local_artifact_path(uri: &str) -> PathBuf {
    if let Some(path) = uri.strip_prefix("file://") {
        return PathBuf::from(path);
    }
    PathBuf::from(uri)
}
