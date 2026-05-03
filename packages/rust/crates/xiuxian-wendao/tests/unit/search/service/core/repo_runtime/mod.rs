use crate::search::SearchCorpusKind;
use crate::search::service::core::types::SearchPlaneService;
use std::fs;

impl SearchPlaneService {
    #[cfg(test)]
    pub(crate) fn clear_in_memory_repo_runtime_for_test(&self, repo_id: &str) {
        self.repo_corpus_records
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|(_, candidate_repo_id), _| candidate_repo_id != repo_id);
    }

    #[cfg(test)]
    pub(crate) fn clear_all_in_memory_repo_corpus_records_for_test(&self) {
        self.repo_corpus_records
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    #[cfg(test)]
    pub(crate) async fn clear_persisted_repo_corpus_for_test(&self, repo_id: &str) {
        self.clear_in_memory_repo_runtime_for_test(repo_id);
        for corpus in [
            SearchCorpusKind::RepoEntity,
            SearchCorpusKind::RepoContentChunk,
        ] {
            self.cache.delete_repo_corpus_record(corpus, repo_id).await;
            let _ = fs::remove_file(self.repo_corpus_record_json_path(corpus, repo_id));
        }
        self.cache.delete_repo_corpus_snapshot().await;
        let _ = fs::remove_file(self.repo_corpus_snapshot_json_path());
    }

    #[cfg(test)]
    pub(crate) fn clear_in_memory_repo_publications_for_test(&self, repo_id: &str) {
        for record in self
            .repo_corpus_records
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values_mut()
        {
            if record.repo_id == repo_id {
                record.publication = None;
            }
        }
    }
}
