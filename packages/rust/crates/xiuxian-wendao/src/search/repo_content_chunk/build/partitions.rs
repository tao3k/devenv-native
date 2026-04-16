use crate::search::SearchFileFingerprint;

pub(crate) const REPO_CONTENT_CHUNK_SMALL_PARTITION_COUNT: usize = 16;
pub(crate) const REPO_CONTENT_CHUNK_LARGE_PARTITION_COUNT: usize = 64;
pub(crate) const REPO_CONTENT_CHUNK_LARGE_REPO_DOCUMENT_THRESHOLD: usize = 4_096;

#[must_use]
pub(crate) fn repo_content_chunk_partition_count_for_document_count(
    document_count: usize,
) -> usize {
    if document_count >= REPO_CONTENT_CHUNK_LARGE_REPO_DOCUMENT_THRESHOLD {
        REPO_CONTENT_CHUNK_LARGE_PARTITION_COUNT
    } else {
        REPO_CONTENT_CHUNK_SMALL_PARTITION_COUNT
    }
}

#[must_use]
pub(crate) fn repo_content_chunk_partition_id_for_count(
    path: &str,
    partition_count: usize,
) -> String {
    let hash = blake3::hash(path.as_bytes());
    let bucket = u16::from_be_bytes([hash.as_bytes()[0], hash.as_bytes()[1]])
        % u16::try_from(partition_count).unwrap_or(u16::MAX);
    format!("{bucket:02}")
}

#[must_use]
pub(crate) fn repo_content_chunk_partition_id_for_path(
    path: &str,
    fingerprints: &std::collections::BTreeMap<String, SearchFileFingerprint>,
    fallback_partition_count: usize,
) -> String {
    fingerprints
        .get(path)
        .and_then(|fingerprint| fingerprint.partition_id.clone())
        .unwrap_or_else(|| {
            repo_content_chunk_partition_id_for_count(path, fallback_partition_count)
        })
}
