use std::collections::{BTreeMap, BTreeSet};

use crate::search::{SearchCorpusKind, SearchFileFingerprint, SearchRepoPublicationRecord};

#[derive(Debug, Clone)]
pub(crate) enum RepoStagedMutationAction<T> {
    Noop,
    RefreshPublication {
        table_name: String,
    },
    ReplaceAll {
        table_name: String,
        payload: T,
    },
    CloneAndMutate {
        base_table_name: String,
        target_table_name: String,
        replaced_paths: BTreeSet<String>,
        changed_payload: T,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct RepoStagedMutationPlan<T> {
    pub(crate) file_fingerprints: BTreeMap<String, SearchFileFingerprint>,
    pub(crate) action: RepoStagedMutationAction<T>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RepoStagedMutationConfig<'a> {
    pub(crate) repo_id: &'a str,
    pub(crate) table_name_prefix: &'a str,
    pub(crate) corpus: SearchCorpusKind,
    pub(crate) extractor_version: u32,
    pub(crate) source_revision: Option<&'a str>,
    pub(crate) previous_publication: Option<&'a SearchRepoPublicationRecord>,
    pub(crate) previous_fingerprints: &'a BTreeMap<String, SearchFileFingerprint>,
}

#[derive(Debug, Clone)]
pub(crate) struct RepoStagedMutationPayload<T> {
    pub(crate) file_fingerprints: BTreeMap<String, SearchFileFingerprint>,
    pub(crate) replace_payload: T,
    pub(crate) changed_payload: T,
    pub(crate) changed_paths: BTreeSet<String>,
    pub(crate) deleted_paths: BTreeSet<String>,
}

#[must_use]
pub(crate) fn plan_repo_staged_mutation<T>(
    config: RepoStagedMutationConfig<'_>,
    payload: RepoStagedMutationPayload<T>,
) -> RepoStagedMutationPlan<T> {
    let RepoStagedMutationConfig {
        repo_id,
        table_name_prefix,
        corpus,
        extractor_version,
        source_revision,
        previous_publication,
        previous_fingerprints,
    } = config;
    let RepoStagedMutationPayload {
        file_fingerprints,
        replace_payload,
        changed_payload,
        changed_paths,
        deleted_paths,
    } = payload;

    let Some(previous_publication) = previous_publication else {
        let table_name = versioned_repo_table_name(
            table_name_prefix,
            repo_id,
            &file_fingerprints,
            source_revision,
            corpus,
            extractor_version,
        );
        return RepoStagedMutationPlan {
            file_fingerprints,
            action: RepoStagedMutationAction::ReplaceAll {
                table_name,
                payload: replace_payload,
            },
        };
    };

    if repo_file_fingerprint_maps_equivalent(previous_fingerprints, &file_fingerprints) {
        return RepoStagedMutationPlan {
            file_fingerprints,
            action: if previous_publication.source_revision.as_deref() == source_revision {
                RepoStagedMutationAction::Noop
            } else {
                RepoStagedMutationAction::RefreshPublication {
                    table_name: previous_publication.table_name.clone(),
                }
            },
        };
    }

    let mut replaced_paths = changed_paths;
    replaced_paths.extend(deleted_paths);
    let target_table_name = versioned_repo_table_name(
        table_name_prefix,
        repo_id,
        &file_fingerprints,
        source_revision,
        corpus,
        extractor_version,
    );
    RepoStagedMutationPlan {
        file_fingerprints,
        action: RepoStagedMutationAction::CloneAndMutate {
            base_table_name: previous_publication.table_name.clone(),
            target_table_name,
            replaced_paths,
            changed_payload,
        },
    }
}

#[must_use]
pub(crate) fn repo_file_fingerprint_changed(
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
    path: &str,
    current: &SearchFileFingerprint,
) -> bool {
    match previous_fingerprints.get(path) {
        Some(previous) => !previous.equivalent_for_incremental(current),
        None => true,
    }
}

#[must_use]
pub(crate) fn repo_file_fingerprint_maps_equivalent(
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
    file_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> bool {
    previous_fingerprints.len() == file_fingerprints.len()
        && file_fingerprints.iter().all(|(path, fingerprint)| {
            !repo_file_fingerprint_changed(previous_fingerprints, path.as_str(), fingerprint)
        })
}

#[must_use]
pub(crate) fn versioned_repo_table_name(
    table_name_prefix: &str,
    repo_id: &str,
    file_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
    source_revision: Option<&str>,
    corpus: SearchCorpusKind,
    extractor_version: u32,
) -> String {
    let normalized_revision = source_revision
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    let mut hasher = blake3::Hasher::new();
    hasher.update(repo_id.as_bytes());
    hasher.update(b"|");
    hasher.update(normalized_revision.as_bytes());
    hasher.update(b"|schema:");
    update_hash_with_number(&mut hasher, corpus.schema_version());
    hasher.update(b"|extractor:");
    update_hash_with_number(&mut hasher, extractor_version);
    for (path, fingerprint) in file_fingerprints {
        hasher.update(b"|");
        hasher.update(path.as_bytes());
        hasher.update(b":");
        hasher.update(
            fingerprint
                .partition_id
                .as_deref()
                .unwrap_or_default()
                .as_bytes(),
        );
        hasher.update(b":");
        if let Some(blake3) = fingerprint.blake3.as_deref() {
            hasher.update(b"semantic:");
            hasher.update(blake3.as_bytes());
        } else {
            hasher.update(b"metadata:");
            update_hash_with_number(&mut hasher, fingerprint.size_bytes);
            hasher.update(b":");
            update_hash_with_number(&mut hasher, fingerprint.modified_unix_ms);
        }
    }
    let token = hasher.finalize().to_hex().to_string();
    format!("{table_name_prefix}_{}", &token[..16])
}

fn update_hash_with_number<N>(hasher: &mut blake3::Hasher, value: N)
where
    N: ToString,
{
    hasher.update(value.to_string().as_bytes());
}

#[cfg(test)]
mod tests {
    use super::versioned_repo_table_name;
    use crate::search::{SearchCorpusKind, SearchFileFingerprint};
    use std::collections::BTreeMap;

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
}
