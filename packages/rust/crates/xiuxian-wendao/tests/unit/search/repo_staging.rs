use std::collections::BTreeMap;

use crate::search::repo_staging::versioned_repo_table_name;
use crate::search::{SearchCorpusKind, SearchFileFingerprint};

#[test]
fn versioned_repo_table_name_matches_legacy_payload_hash_semantics() {
    let file_fingerprints = BTreeMap::from([
        (
            "src/alpha.jl".to_string(),
            SearchFileFingerprint {
                relative_path: "src/alpha.jl".to_string(),
                partition_id: Some("p-01".to_string()),
                size_bytes: 128,
                modified_unix_ms: 11,
                extractor_version: 1,
                schema_version: 4,
                blake3: Some("semantic-alpha".to_string()),
            },
        ),
        (
            "src/beta.jl".to_string(),
            SearchFileFingerprint {
                relative_path: "src/beta.jl".to_string(),
                partition_id: None,
                size_bytes: 256,
                modified_unix_ms: 22,
                extractor_version: 1,
                schema_version: 4,
                blake3: None,
            },
        ),
    ]);

    let actual = versioned_repo_table_name(
        "repo_content_chunk_alpha_repo",
        "alpha/repo",
        &file_fingerprints,
        Some("  REV-2  "),
        SearchCorpusKind::RepoContentChunk,
        7,
    );

    assert_eq!(
        actual,
        legacy_versioned_repo_table_name(
            "repo_content_chunk_alpha_repo",
            "alpha/repo",
            &file_fingerprints,
            Some("  REV-2  "),
            SearchCorpusKind::RepoContentChunk,
            7,
        )
    );
}

fn legacy_versioned_repo_table_name(
    table_name_prefix: &str,
    repo_id: &str,
    file_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
    source_revision: Option<&str>,
    corpus: SearchCorpusKind,
    extractor_version: u32,
) -> String {
    let mut payload = format!(
        "{repo_id}|{}|schema:{}|extractor:{}",
        source_revision
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase(),
        corpus.schema_version(),
        extractor_version,
    );
    for (path, fingerprint) in file_fingerprints {
        payload.push('|');
        payload.push_str(path.as_str());
        payload.push(':');
        payload.push_str(fingerprint.partition_id.as_deref().unwrap_or_default());
        payload.push(':');
        if let Some(blake3) = fingerprint.blake3.as_deref() {
            payload.push_str("semantic:");
            payload.push_str(blake3);
        } else {
            payload.push_str("metadata:");
            payload.push_str(fingerprint.size_bytes.to_string().as_str());
            payload.push(':');
            payload.push_str(fingerprint.modified_unix_ms.to_string().as_str());
        }
    }
    let token = blake3::hash(payload.as_bytes()).to_hex().to_string();
    format!("{table_name_prefix}_{}", &token[..16])
}
