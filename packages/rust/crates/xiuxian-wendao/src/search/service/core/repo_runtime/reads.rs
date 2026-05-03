use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use crate::search::service::core::types::{RepoRuntimeState, SearchPlaneService};
use crate::search::{SearchCorpusKind, SearchRepoCorpusRecord};

impl SearchPlaneService {
    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    pub fn clear_all_in_memory_repo_runtime_for_test(&self) {
        self.repo_corpus_records
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    fn merge_persisted_repo_corpus_record(
        current: &mut SearchRepoCorpusRecord,
        persisted: SearchRepoCorpusRecord,
    ) -> bool {
        let mut changed = false;
        if current.publication.is_none() && persisted.publication.is_some() {
            current.publication = persisted.publication;
            changed = true;
        }
        if current.maintenance.is_none() && persisted.maintenance.is_some() {
            current.maintenance = persisted.maintenance;
            changed = true;
        }
        changed
    }

    async fn recover_persisted_repo_corpus_record_for_reads(
        &self,
        record: SearchRepoCorpusRecord,
    ) -> (SearchRepoCorpusRecord, bool) {
        let (mut record, mut changed) = self.reconcile_repo_corpus_record_for_reads(record);
        if record.publication.is_some() {
            return (record, changed);
        }

        if let Some(cache_record) = self
            .cache
            .get_repo_corpus_record(record.corpus, record.repo_id.as_str())
            .await
        {
            let (cache_record, cache_changed) =
                self.reconcile_repo_corpus_record_for_reads(cache_record);
            changed |= cache_changed;
            changed |= Self::merge_persisted_repo_corpus_record(&mut record, cache_record);
        }

        if record.publication.is_none()
            && let Some(local_record) =
                self.load_local_repo_corpus_record(record.corpus, record.repo_id.as_str())
        {
            let (local_record, local_changed) =
                self.reconcile_repo_corpus_record_for_reads(local_record);
            changed |= local_changed;
            changed |= Self::merge_persisted_repo_corpus_record(&mut record, local_record);
        }

        (record, changed)
    }

    #[cfg(test)]
    pub(crate) async fn repo_search_publication_state(
        &self,
        repo_id: &str,
    ) -> crate::search::service::core::types::RepoSearchPublicationState {
        let entity_record = self
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoEntity, repo_id)
            .await;
        let content_record = self
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, repo_id)
            .await;
        Self::repo_search_publication_state_from_records(
            entity_record.as_ref(),
            content_record.as_ref(),
        )
    }

    pub(crate) async fn repo_search_publication_states(
        &self,
        repo_ids: &[String],
    ) -> BTreeMap<String, crate::search::service::core::types::RepoSearchPublicationState> {
        let records = self.repo_corpus_records_for_repo_ids(repo_ids).await;
        repo_ids
            .iter()
            .map(|repo_id| {
                let entity_record = records.get(&(SearchCorpusKind::RepoEntity, repo_id.clone()));
                let content_record =
                    records.get(&(SearchCorpusKind::RepoContentChunk, repo_id.clone()));
                (
                    repo_id.clone(),
                    Self::repo_search_publication_state_from_records(entity_record, content_record),
                )
            })
            .collect()
    }

    pub(crate) fn repo_runtime_state(&self, repo_id: &str) -> Option<RepoRuntimeState> {
        self.current_repo_runtime_states().remove(repo_id)
    }

    pub(crate) fn cached_repo_publication(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
    ) -> Option<crate::search::SearchRepoPublicationRecord> {
        self.repo_corpus_records
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&(corpus, repo_id.to_string()))
            .and_then(|record| record.publication.clone())
    }

    fn cached_repo_corpus_record(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
    ) -> Option<SearchRepoCorpusRecord> {
        self.repo_corpus_records
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&(corpus, repo_id.to_string()))
            .cloned()
    }

    fn reconcile_repo_corpus_record(
        &self,
        mut record: SearchRepoCorpusRecord,
    ) -> (SearchRepoCorpusRecord, bool) {
        let mut changed = false;
        if let Some(runtime) = self.repo_runtime_state(record.repo_id.as_str()) {
            let runtime_record = Self::runtime_record_from_state(record.repo_id.as_str(), &runtime);
            if record.runtime.as_ref() != Some(&runtime_record) {
                record.runtime = Some(runtime_record);
                changed = true;
            }
        }
        if let Some(publication) =
            self.cached_repo_publication(record.corpus, record.repo_id.as_str())
            && record.publication.as_ref() != Some(&publication)
        {
            record.publication = Some(publication);
            changed = true;
        }
        (record, changed)
    }

    fn reconcile_repo_corpus_record_for_reads(
        &self,
        record: SearchRepoCorpusRecord,
    ) -> (SearchRepoCorpusRecord, bool) {
        self.reconcile_repo_corpus_record(record)
    }

    async fn reconcile_repo_corpus_records_for_reads(
        &self,
        records: BTreeMap<(SearchCorpusKind, String), SearchRepoCorpusRecord>,
    ) -> BTreeMap<(SearchCorpusKind, String), SearchRepoCorpusRecord> {
        let mut changed_records = Vec::new();
        let mut reconciled = BTreeMap::new();
        for (key, record) in records {
            let (record, changed) = self
                .recover_persisted_repo_corpus_record_for_reads(record)
                .await;
            if changed {
                changed_records.push(record.clone());
            }
            reconciled.insert(key, record);
        }
        for record in &changed_records {
            self.persist_local_repo_corpus_record(record);
            self.cache.set_repo_corpus_record(record).await;
        }
        reconciled
    }

    fn repo_corpus_record_keys_for_repo_ids(
        repo_ids: &BTreeSet<String>,
    ) -> Vec<(SearchCorpusKind, String)> {
        repo_ids
            .iter()
            .flat_map(|repo_id| {
                [
                    SearchCorpusKind::RepoEntity,
                    SearchCorpusKind::RepoContentChunk,
                ]
                .into_iter()
                .map(move |corpus| (corpus, repo_id.clone()))
            })
            .collect()
    }

    fn missing_repo_corpus_record_keys(
        records: &BTreeMap<(SearchCorpusKind, String), SearchRepoCorpusRecord>,
        repo_ids: &BTreeSet<String>,
    ) -> Vec<(SearchCorpusKind, String)> {
        Self::repo_corpus_record_keys_for_repo_ids(repo_ids)
            .into_iter()
            .filter(|key| !records.contains_key(key))
            .collect()
    }

    pub(super) fn filter_repo_corpus_records(
        records: BTreeMap<(SearchCorpusKind, String), SearchRepoCorpusRecord>,
        repo_ids: &BTreeSet<String>,
    ) -> BTreeMap<(SearchCorpusKind, String), SearchRepoCorpusRecord> {
        if repo_ids.is_empty() {
            return records;
        }
        records
            .into_iter()
            .filter(|(_, record)| repo_ids.contains(&record.repo_id))
            .collect()
    }

    async fn repo_corpus_records_for_repo_ids(
        &self,
        repo_ids: &[String],
    ) -> BTreeMap<(SearchCorpusKind, String), SearchRepoCorpusRecord> {
        let repo_ids = repo_ids.iter().cloned().collect::<BTreeSet<_>>();
        if repo_ids.is_empty() {
            return BTreeMap::new();
        }

        let mut records = Self::filter_repo_corpus_records(
            self.repo_corpus_records
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
            &repo_ids,
        );
        let mut missing_keys = Self::missing_repo_corpus_record_keys(&records, &repo_ids);
        if !missing_keys.is_empty() {
            let cached_records = self
                .reconcile_repo_corpus_records_for_reads(
                    self.cache
                        .get_repo_corpus_records(missing_keys.as_slice())
                        .await,
                )
                .await;
            if !cached_records.is_empty() {
                self.repo_corpus_records
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .extend(cached_records.clone());
                records.extend(cached_records);
            }
            missing_keys = Self::missing_repo_corpus_record_keys(&records, &repo_ids);
        }
        if !missing_keys.is_empty() {
            let local_records = self
                .reconcile_repo_corpus_records_for_reads(
                    self.load_local_repo_corpus_records(missing_keys.as_slice()),
                )
                .await;
            if !local_records.is_empty() {
                self.repo_corpus_records
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .extend(local_records.clone());
                records.extend(local_records);
            }
        }
        records
    }

    pub(crate) async fn repo_corpus_snapshot_for_reads(
        &self,
    ) -> BTreeMap<(SearchCorpusKind, String), SearchRepoCorpusRecord> {
        let mut records = self
            .repo_corpus_records
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for (key, record) in self.load_local_repo_corpus_record_inventory() {
            records.entry(key).or_insert(record);
        }
        if records.is_empty() {
            return BTreeMap::new();
        }
        let records = self.reconcile_repo_corpus_records_for_reads(records).await;
        *self
            .repo_corpus_records
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = records.clone();
        records
    }

    /// Return a repository corpus record reconciled for read-side consumers.
    pub async fn repo_corpus_record_for_reads(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
    ) -> Option<SearchRepoCorpusRecord> {
        if let Some(record) = self.cached_repo_corpus_record(corpus, repo_id) {
            let (record, changed) = self
                .recover_persisted_repo_corpus_record_for_reads(record)
                .await;
            if changed {
                self.repo_corpus_records
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .insert((corpus, repo_id.to_string()), record.clone());
                self.persist_local_repo_corpus_record(&record);
                self.cache.set_repo_corpus_record(&record).await;
            }
            return Some(record);
        }

        let repo_key = (corpus, repo_id.to_string());
        if let Some(record) = self
            .cache
            .get_repo_corpus_records(std::slice::from_ref(&repo_key))
            .await
            .remove(&repo_key)
        {
            let (record, changed) = self
                .recover_persisted_repo_corpus_record_for_reads(record)
                .await;
            self.repo_corpus_records
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert((corpus, repo_id.to_string()), record.clone());
            self.persist_local_repo_corpus_record(&record);
            if changed {
                self.cache.set_repo_corpus_record(&record).await;
            }
            return Some(record);
        }

        if let Some(record) = self.load_local_repo_corpus_record(corpus, repo_id) {
            let (record, changed) = self
                .recover_persisted_repo_corpus_record_for_reads(record)
                .await;
            self.repo_corpus_records
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert((corpus, repo_id.to_string()), record.clone());
            if changed {
                self.persist_local_repo_corpus_record(&record);
                self.cache.set_repo_corpus_record(&record).await;
            }
            return Some(record);
        }
        None
    }

    fn load_local_repo_corpus_records(
        &self,
        keys: &[(SearchCorpusKind, String)],
    ) -> BTreeMap<(SearchCorpusKind, String), SearchRepoCorpusRecord> {
        let mut records = BTreeMap::new();
        for (corpus, repo_id) in keys {
            if let Some(record) = self.load_local_repo_corpus_record(*corpus, repo_id.as_str()) {
                records.insert((*corpus, repo_id.clone()), record);
            }
        }
        records
    }

    fn load_local_repo_corpus_record_inventory(
        &self,
    ) -> BTreeMap<(SearchCorpusKind, String), SearchRepoCorpusRecord> {
        let mut records = BTreeMap::new();
        for corpus in [
            SearchCorpusKind::RepoEntity,
            SearchCorpusKind::RepoContentChunk,
        ] {
            let record_root = self
                .repo_corpus_runtime_root()
                .join("records")
                .join(corpus.as_str());
            let Ok(entries) = fs::read_dir(record_root) else {
                continue;
            };
            for entry in entries.flatten() {
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                if !file_type.is_file() {
                    continue;
                }
                let Ok(payload) = fs::read(entry.path()) else {
                    continue;
                };
                let Ok(record) = serde_json::from_slice::<SearchRepoCorpusRecord>(&payload) else {
                    continue;
                };
                if record.corpus != corpus {
                    continue;
                }
                records.insert((corpus, record.repo_id.clone()), record);
            }
        }
        records
    }

    pub(crate) fn load_local_repo_corpus_record(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
    ) -> Option<SearchRepoCorpusRecord> {
        let payload = fs::read(self.repo_corpus_record_json_path(corpus, repo_id)).ok()?;
        serde_json::from_slice(payload.as_slice()).ok()
    }
}
