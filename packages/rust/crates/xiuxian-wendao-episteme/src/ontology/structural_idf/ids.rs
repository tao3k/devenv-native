use std::path::Path;

use sha2::{Digest, Sha256};

pub(super) fn stable_document_id(file_id: &str) -> String {
    if is_safe_token(file_id) {
        format!("idf.document.{file_id}")
    } else {
        stable_id("idf.document", file_id)
    }
}

pub(super) fn stable_id(prefix: &str, seed: &str) -> String {
    format!("{prefix}.{}", short_hash(seed))
}

pub(super) fn repo_relative_path(episteme_root: &Path, path: &Path) -> String {
    path.strip_prefix(episteme_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn short_hash(seed: &str) -> String {
    let digest = Sha256::digest(seed.as_bytes());
    format!("{digest:x}").chars().take(16).collect()
}

fn is_safe_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}
