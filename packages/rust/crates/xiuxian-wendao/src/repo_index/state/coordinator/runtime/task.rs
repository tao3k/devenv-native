use std::sync::Arc;
use std::time::{Duration, Instant};

use super::incremental::PreparedIncrementalAnalysis;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::{RepoSourceKind, RepoSyncResult};
use crate::repo_index::state::collect::IncrementalCodeDocumentCollection;
use crate::repo_index::state::coordinator::RepoIndexCoordinator;
use crate::repo_index::state::task::{
    RepoIndexTask, RepoTaskFeedback, RepoTaskOutcome, should_retry_sync_failure,
};
use crate::repo_index::types::RepoIndexPhase;
use crate::search::{SearchCorpusKind, SearchPlaneFileFingerprintScope};

enum CodeDocumentsForIndexing {
    Full(Vec<crate::repo_index::types::RepoCodeDocument>),
    Incremental(IncrementalCodeDocumentCollection),
}

struct CompleteIndexingInput {
    repo_id: String,
    started_at: Instant,
    control_elapsed: Duration,
    previous_revision: Option<String>,
    task: RepoIndexTask,
    sync_result: RepoSyncResult,
    analysis: crate::analyzers::RepositoryAnalysisOutput,
}

impl CodeDocumentsForIndexing {
    fn documents(&self) -> &[crate::repo_index::types::RepoCodeDocument] {
        match self {
            Self::Full(documents) => documents.as_slice(),
            Self::Incremental(collection) => collection.changed_documents.as_slice(),
        }
    }
}

impl RepoIndexCoordinator {
    pub(crate) async fn process_task(self: Arc<Self>, task: RepoIndexTask) -> RepoTaskFeedback {
        let repo_id = task.repository.id.clone();
        let started_at = Instant::now();
        let previous_revision = self.previous_revision(repo_id.as_str());
        if !self.fingerprint_matches(repo_id.as_str(), task.fingerprint.as_str()) {
            return repo_task_feedback(repo_id, started_at, None, RepoTaskOutcome::Skipped);
        }

        let (sync_result, control_elapsed) = match self
            .synchronize_task(repo_id.as_str(), &task, started_at)
            .await
        {
            Ok(result) => result,
            Err(feedback) => return feedback,
        };

        if self
            .repo_publications_are_current(repo_id.as_str(), &sync_result)
            .await
        {
            return repo_task_feedback(
                repo_id,
                started_at,
                Some(control_elapsed),
                RepoTaskOutcome::Success {
                    revision: sync_result.revision,
                },
            );
        }

        let analysis = match self
            .resolve_analysis(
                repo_id.as_str(),
                &task,
                &sync_result,
                previous_revision.as_deref(),
                started_at,
                control_elapsed,
            )
            .await
        {
            AnalysisResolution::Finished(feedback) => return *feedback,
            AnalysisResolution::Analysis(analysis) => analysis,
        };

        self.complete_indexing(CompleteIndexingInput {
            repo_id,
            started_at,
            control_elapsed,
            previous_revision,
            task,
            sync_result,
            analysis: *analysis,
        })
        .await
    }

    pub(crate) async fn repo_publications_are_current(
        &self,
        repo_id: &str,
        sync_result: &RepoSyncResult,
    ) -> bool {
        let Some(revision) = sync_result.revision.as_deref() else {
            return false;
        };
        if sync_result.source_kind != RepoSourceKind::ManagedRemote {
            return false;
        }
        self.search_plane
            .repo_backed_publications_are_current_for_revision(repo_id, revision)
            .await
    }

    fn previous_revision(&self, repo_id: &str) -> Option<String> {
        self.statuses
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(repo_id)
            .and_then(|status| status.last_revision.clone())
    }

    async fn synchronize_task(
        &self,
        repo_id: &str,
        task: &RepoIndexTask,
        started_at: Instant,
    ) -> Result<(RepoSyncResult, Duration), RepoTaskFeedback> {
        self.bump_status(repo_id, RepoIndexPhase::Checking, None, None);

        let sync_result = match self
            .run_repository_sync(repo_id, task.repository.clone(), task.refresh)
            .await
        {
            Ok(result) => result,
            Err(error) if should_retry_sync_failure(&error, task.retry_count) => {
                let mut retry_task = task.clone();
                retry_task.retry_count = retry_task.retry_count.saturating_add(1);
                return Err(repo_task_feedback(
                    repo_id.to_string(),
                    started_at,
                    None,
                    RepoTaskOutcome::Requeued {
                        task: retry_task,
                        error,
                    },
                ));
            }
            Err(error) => {
                return Err(repo_task_feedback(
                    repo_id.to_string(),
                    started_at,
                    None,
                    RepoTaskOutcome::Failure {
                        revision: None,
                        error,
                    },
                ));
            }
        };
        let control_elapsed = started_at.elapsed();

        if !self.fingerprint_matches(repo_id, task.fingerprint.as_str()) {
            return Err(repo_task_feedback(
                repo_id.to_string(),
                started_at,
                Some(control_elapsed),
                RepoTaskOutcome::Skipped,
            ));
        }

        self.bump_status(
            repo_id,
            RepoIndexPhase::Indexing,
            sync_result.revision.clone(),
            None,
        );

        Ok((sync_result, control_elapsed))
    }

    async fn resolve_analysis(
        &self,
        repo_id: &str,
        task: &RepoIndexTask,
        sync_result: &RepoSyncResult,
        previous_revision: Option<&str>,
        started_at: Instant,
        control_elapsed: Duration,
    ) -> AnalysisResolution {
        match self.prepare_incremental_analysis(&task.repository, sync_result, previous_revision) {
            Ok(Some(PreparedIncrementalAnalysis::RefreshOnly)) => {
                if self
                    .search_plane
                    .refresh_repo_backed_publications_for_revision(
                        repo_id,
                        sync_result.revision.as_deref().unwrap_or_default(),
                    )
                    .await
                {
                    return AnalysisResolution::Finished(Box::new(repo_task_feedback(
                        repo_id.to_string(),
                        started_at,
                        Some(control_elapsed),
                        RepoTaskOutcome::Success {
                            revision: sync_result.revision.clone(),
                        },
                    )));
                }
            }
            Ok(Some(PreparedIncrementalAnalysis::Analysis(analysis))) => {
                return AnalysisResolution::Analysis(analysis);
            }
            Ok(None) => {}
            Err(error) => {
                return AnalysisResolution::Finished(Box::new(repo_task_feedback(
                    repo_id.to_string(),
                    started_at,
                    Some(control_elapsed),
                    RepoTaskOutcome::Failure {
                        revision: sync_result.revision.clone(),
                        error,
                    },
                )));
            }
        }

        match self.run_repository_analysis(task.repository.clone()).await {
            Ok(analysis) => AnalysisResolution::Analysis(Box::new(analysis)),
            Err(error) => AnalysisResolution::Finished(Box::new(repo_task_feedback(
                repo_id.to_string(),
                started_at,
                Some(control_elapsed),
                RepoTaskOutcome::Failure {
                    revision: sync_result.revision.clone(),
                    error,
                },
            ))),
        }
    }

    async fn complete_indexing(&self, input: CompleteIndexingInput) -> RepoTaskFeedback {
        let CompleteIndexingInput {
            repo_id,
            started_at,
            control_elapsed,
            previous_revision,
            task,
            sync_result,
            analysis,
        } = input;
        if !self.fingerprint_matches(repo_id.as_str(), task.fingerprint.as_str()) {
            return repo_task_feedback(
                repo_id,
                started_at,
                Some(control_elapsed),
                RepoTaskOutcome::Skipped,
            );
        }

        let code_documents = match self
            .collect_code_documents_for_indexing(
                repo_id.as_str(),
                started_at,
                control_elapsed,
                previous_revision.as_deref(),
                &task,
                &sync_result,
            )
            .await
        {
            Ok(code_documents) => code_documents,
            Err(feedback) => return feedback,
        };

        if !self.fingerprint_matches(repo_id.as_str(), task.fingerprint.as_str()) {
            return repo_task_feedback(
                repo_id,
                started_at,
                Some(control_elapsed),
                RepoTaskOutcome::Skipped,
            );
        }

        if let Err(feedback) = self
            .publish_repo_corpora(
                repo_id.as_str(),
                started_at,
                control_elapsed,
                &sync_result,
                &analysis,
                &code_documents,
            )
            .await
        {
            return feedback;
        }

        repo_task_feedback(
            repo_id,
            started_at,
            Some(control_elapsed),
            RepoTaskOutcome::Success {
                revision: sync_result.revision,
            },
        )
    }

    async fn collect_code_documents_for_indexing(
        &self,
        repo_id: &str,
        started_at: Instant,
        control_elapsed: Duration,
        previous_revision: Option<&str>,
        task: &RepoIndexTask,
        sync_result: &RepoSyncResult,
    ) -> Result<CodeDocumentsForIndexing, RepoTaskFeedback> {
        if let Some(incremental) = self
            .try_collect_incremental_code_documents_for_indexing(
                repo_id,
                task,
                sync_result,
                previous_revision,
            )
            .await
        {
            return match incremental {
                Ok(Some(collection)) => Ok(CodeDocumentsForIndexing::Incremental(collection)),
                Ok(None) => Err(repo_task_feedback(
                    repo_id.to_string(),
                    started_at,
                    Some(control_elapsed),
                    RepoTaskOutcome::Skipped,
                )),
                Err(_) => self
                    .collect_full_code_documents_for_indexing(
                        repo_id,
                        started_at,
                        control_elapsed,
                        task,
                        sync_result,
                    )
                    .await
                    .map(CodeDocumentsForIndexing::Full),
            };
        }

        self.collect_full_code_documents_for_indexing(
            repo_id,
            started_at,
            control_elapsed,
            task,
            sync_result,
        )
        .await
        .map(CodeDocumentsForIndexing::Full)
    }

    async fn collect_full_code_documents_for_indexing(
        &self,
        repo_id: &str,
        started_at: Instant,
        control_elapsed: Duration,
        task: &RepoIndexTask,
        sync_result: &RepoSyncResult,
    ) -> Result<Vec<crate::repo_index::types::RepoCodeDocument>, RepoTaskFeedback> {
        match self
            .collect_code_documents_for_task(
                repo_id,
                task.fingerprint.as_str(),
                sync_result.checkout_path.as_str(),
            )
            .await
        {
            Ok(Some(code_documents)) => Ok(code_documents),
            Ok(None) => Err(repo_task_feedback(
                repo_id.to_string(),
                started_at,
                Some(control_elapsed),
                RepoTaskOutcome::Skipped,
            )),
            Err(error) => Err(repo_task_feedback(
                repo_id.to_string(),
                started_at,
                Some(control_elapsed),
                RepoTaskOutcome::Failure {
                    revision: sync_result.revision.clone(),
                    error,
                },
            )),
        }
    }

    async fn try_collect_incremental_code_documents_for_indexing(
        &self,
        repo_id: &str,
        task: &RepoIndexTask,
        sync_result: &RepoSyncResult,
        previous_revision: Option<&str>,
    ) -> Option<Result<Option<IncrementalCodeDocumentCollection>, RepoIntelligenceError>> {
        let current_revision = sync_result.revision.as_deref()?;
        let previous_revision =
            previous_revision.filter(|revision| *revision != current_revision)?;
        let current_record = self
            .search_plane
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, repo_id)
            .await?;
        let previous_publication = current_record.publication.as_ref()?;
        let previous_fingerprints = self
            .search_plane
            .file_fingerprints(SearchPlaneFileFingerprintScope::repo_corpus(
                SearchCorpusKind::RepoContentChunk,
                repo_id,
            ))
            .await;
        if previous_fingerprints.is_empty() && previous_publication.row_count != 0 {
            return None;
        }
        Some(
            self.collect_incremental_code_documents_for_task(
                repo_id,
                task.fingerprint.as_str(),
                sync_result.checkout_path.as_str(),
                previous_revision,
                current_revision,
            )
            .await,
        )
    }

    async fn publish_repo_corpora(
        &self,
        repo_id: &str,
        started_at: Instant,
        control_elapsed: Duration,
        sync_result: &RepoSyncResult,
        analysis: &crate::analyzers::RepositoryAnalysisOutput,
        code_documents: &CodeDocumentsForIndexing,
    ) -> Result<(), RepoTaskFeedback> {
        if let Err(error) = self
            .search_plane
            .publish_repo_entities_with_revision(
                repo_id,
                analysis,
                code_documents.documents(),
                sync_result.revision.as_deref(),
            )
            .await
        {
            return Err(repo_task_analysis_failure(
                repo_id,
                started_at,
                control_elapsed,
                sync_result.revision.clone(),
                format!("repo `{repo_id}` repo-entity publish failed: {error}"),
            ));
        }

        let repo_content_result = match code_documents {
            CodeDocumentsForIndexing::Full(documents) => {
                self.search_plane
                    .publish_repo_content_chunks_with_revision(
                        repo_id,
                        documents,
                        sync_result.revision.as_deref(),
                    )
                    .await
            }
            CodeDocumentsForIndexing::Incremental(collection) => {
                self.search_plane
                    .publish_repo_content_chunks_incremental_with_revision(
                        repo_id,
                        collection.changed_documents.as_slice(),
                        &collection.deleted_paths,
                        sync_result.revision.as_deref(),
                    )
                    .await
            }
        };
        if let Err(error) = repo_content_result {
            return Err(repo_task_analysis_failure(
                repo_id,
                started_at,
                control_elapsed,
                sync_result.revision.clone(),
                format!("repo `{repo_id}` repo-content chunk publish failed: {error}"),
            ));
        }

        Ok(())
    }
}

fn repo_task_feedback(
    repo_id: String,
    started_at: Instant,
    control_elapsed: Option<Duration>,
    outcome: RepoTaskOutcome,
) -> RepoTaskFeedback {
    let elapsed = started_at.elapsed();
    match control_elapsed {
        Some(control_elapsed) => {
            RepoTaskFeedback::with_control_elapsed(repo_id, elapsed, control_elapsed, outcome)
        }
        None => RepoTaskFeedback::new(repo_id, elapsed, outcome),
    }
}

fn repo_task_analysis_failure(
    repo_id: &str,
    started_at: Instant,
    control_elapsed: Duration,
    revision: Option<String>,
    message: String,
) -> RepoTaskFeedback {
    repo_task_feedback(
        repo_id.to_string(),
        started_at,
        Some(control_elapsed),
        RepoTaskOutcome::Failure {
            revision,
            error: RepoIntelligenceError::AnalysisFailed { message },
        },
    )
}

enum AnalysisResolution {
    Finished(Box<RepoTaskFeedback>),
    Analysis(Box<crate::analyzers::RepositoryAnalysisOutput>),
}
