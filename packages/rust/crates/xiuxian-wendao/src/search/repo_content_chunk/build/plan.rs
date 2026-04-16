use std::collections::{BTreeMap, BTreeSet};

use crate::repo_index::RepoCodeDocument;
use crate::search::repo_content_chunk::build::types::{
    REPO_CONTENT_CHUNK_EXTRACTOR_VERSION, RepoContentChunkBuildPlan,
};
use crate::search::repo_staging::{
    RepoStagedMutationConfig, RepoStagedMutationPayload, repo_file_fingerprint_changed,
};
use crate::search::{
    SearchCorpusKind, SearchFileFingerprint, SearchPlaneService, plan_repo_staged_mutation,
    stable_payload_fingerprint,
};

pub(crate) fn plan_repo_content_chunk_build(
    repo_id: &str,
    documents: &[RepoCodeDocument],
    source_revision: Option<&str>,
    previous_publication: Option<&crate::search::SearchRepoPublicationRecord>,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> RepoContentChunkBuildPlan {
    let file_fingerprints = repo_content_chunk_file_fingerprints(documents);

    let changed_documents = documents
        .iter()
        .filter(|document| {
            file_fingerprints
                .get(document.path.as_str())
                .is_some_and(|fingerprint| {
                    repo_file_fingerprint_changed(
                        previous_fingerprints,
                        document.path.as_str(),
                        fingerprint,
                    )
                })
        })
        .cloned()
        .collect::<Vec<_>>();
    let changed_paths = changed_documents
        .iter()
        .map(|document| document.path.clone())
        .collect::<BTreeSet<_>>();
    let deleted_paths = previous_fingerprints
        .keys()
        .filter(|path| !file_fingerprints.contains_key(*path))
        .cloned()
        .collect::<BTreeSet<_>>();

    plan_repo_staged_mutation(
        RepoStagedMutationConfig {
            repo_id,
            table_name_prefix: SearchPlaneService::repo_content_chunk_table_name(repo_id).as_str(),
            corpus: SearchCorpusKind::RepoContentChunk,
            extractor_version: REPO_CONTENT_CHUNK_EXTRACTOR_VERSION,
            source_revision,
            previous_publication,
            previous_fingerprints,
        },
        RepoStagedMutationPayload {
            file_fingerprints,
            replace_payload: documents.to_vec(),
            changed_payload: changed_documents,
            changed_paths,
            deleted_paths,
        },
    )
}

pub(crate) fn plan_repo_content_chunk_incremental_build(
    repo_id: &str,
    changed_documents: &[RepoCodeDocument],
    file_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
    source_revision: Option<&str>,
    previous_publication: Option<&crate::search::SearchRepoPublicationRecord>,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> RepoContentChunkBuildPlan {
    let changed_paths = file_fingerprints
        .iter()
        .filter_map(|(path, fingerprint)| {
            repo_file_fingerprint_changed(previous_fingerprints, path.as_str(), fingerprint)
                .then_some(path.clone())
        })
        .collect::<BTreeSet<_>>();
    let changed_documents = changed_documents
        .iter()
        .filter(|document| changed_paths.contains(document.path.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let deleted_paths = previous_fingerprints
        .keys()
        .filter(|path| !file_fingerprints.contains_key(*path))
        .cloned()
        .collect::<BTreeSet<_>>();

    plan_repo_staged_mutation(
        RepoStagedMutationConfig {
            repo_id,
            table_name_prefix: SearchPlaneService::repo_content_chunk_table_name(repo_id).as_str(),
            corpus: SearchCorpusKind::RepoContentChunk,
            extractor_version: REPO_CONTENT_CHUNK_EXTRACTOR_VERSION,
            source_revision,
            previous_publication,
            previous_fingerprints,
        },
        RepoStagedMutationPayload {
            file_fingerprints: file_fingerprints.clone(),
            replace_payload: changed_documents.clone(),
            changed_payload: changed_documents,
            changed_paths,
            deleted_paths,
        },
    )
}

pub(crate) fn merge_repo_content_chunk_file_fingerprints(
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
    changed_documents: &[RepoCodeDocument],
    deleted_paths: &BTreeSet<String>,
) -> BTreeMap<String, SearchFileFingerprint> {
    let mut file_fingerprints = previous_fingerprints.clone();
    for path in deleted_paths {
        file_fingerprints.remove(path);
    }
    file_fingerprints.extend(repo_content_chunk_file_fingerprints(changed_documents));
    file_fingerprints
}

pub(crate) fn repo_content_chunk_file_fingerprints(
    documents: &[RepoCodeDocument],
) -> BTreeMap<String, SearchFileFingerprint> {
    documents
        .iter()
        .map(|document| {
            let mut fingerprint = document.to_file_fingerprint(
                REPO_CONTENT_CHUNK_EXTRACTOR_VERSION,
                SearchCorpusKind::RepoContentChunk.schema_version(),
            );
            fingerprint.blake3 = Some(stable_payload_fingerprint(
                "repo_content_chunk_document",
                document.contents.as_ref(),
            ));
            (document.path.clone(), fingerprint)
        })
        .collect::<BTreeMap<_, _>>()
}

#[cfg(test)]
pub(crate) fn versioned_repo_content_table_name(
    repo_id: &str,
    file_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
    source_revision: Option<&str>,
) -> String {
    crate::search::repo_staging::versioned_repo_table_name(
        SearchPlaneService::repo_content_chunk_table_name(repo_id).as_str(),
        repo_id,
        file_fingerprints,
        source_revision,
        SearchCorpusKind::RepoContentChunk,
        REPO_CONTENT_CHUNK_EXTRACTOR_VERSION,
    )
}
