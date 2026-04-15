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
        return RepoStagedMutationPlan {
            file_fingerprints: file_fingerprints.clone(),
            action: RepoStagedMutationAction::ReplaceAll {
                table_name: versioned_repo_table_name(
                    table_name_prefix,
                    repo_id,
                    &file_fingerprints,
                    source_revision,
                    corpus,
                    extractor_version,
                ),
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
    RepoStagedMutationPlan {
        file_fingerprints: file_fingerprints.clone(),
        action: RepoStagedMutationAction::CloneAndMutate {
            base_table_name: previous_publication.table_name.clone(),
            target_table_name: versioned_repo_table_name(
                table_name_prefix,
                repo_id,
                &file_fingerprints,
                source_revision,
                corpus,
                extractor_version,
            ),
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
