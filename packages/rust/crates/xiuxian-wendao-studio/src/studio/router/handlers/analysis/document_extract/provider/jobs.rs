use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use arrow::array::{Array, StringArray};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_server::transport::{
    DocumentExtractFlightRequest, DocumentExtractFlightRouteResponse,
};

use super::StudioDocumentExtractFlightRouteProvider;
use crate::studio::router::handlers::analysis::document_extract::arrow_cache::{
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME, build_error_resource_batch, build_job_resource_batch,
    mirror_artifact_to_output, mirror_document_extract_cache, read_arrow_file,
    read_cached_document_batches, write_arrow_file,
};
use crate::studio::router::handlers::analysis::document_extract::registry::{
    DocumentExtractJobRegistry, DocumentExtractJobStatus, artifact_ready, default_output_dir,
};

impl StudioDocumentExtractFlightRouteProvider {
    pub(super) async fn sync_document_extract_batch(
        &self,
        source_path: &str,
        output_dir: &str,
        force: bool,
        error_row: bool,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let source = PathBuf::from(source_path);
        let output = if output_dir.trim().is_empty() {
            default_output_dir(source.as_path())
        } else {
            PathBuf::from(output_dir)
        };
        if source.exists() && !force {
            if let Some(batches) = read_cached_document_batches(source.as_path(), output.as_path())?
            {
                return Ok(DocumentExtractFlightRouteResponse::from_batches(batches));
            }
            match self
                .reuse_succeeded_artifact(source.as_path(), output.as_path())
                .await
            {
                Ok(Some(response)) => return Ok(response),
                Ok(None) => {}
                Err(error) => {
                    log::warn!("failed to reuse sync document extract artifact: {error}");
                }
            }
        }

        let output_string = output.to_string_lossy().to_string();
        let engine_batches = self
            .request_python_document_extract(source_path, output_string.as_str(), force, error_row)
            .await?;
        if source.exists()
            && document_extract_batches_are_cacheable(engine_batches.as_slice())
            && let Err(error) = self
                .persist_sync_output_artifact(source.as_path(), output.as_path())
                .await
        {
            log::warn!("failed to persist sync document extract artifact: {error}");
        }
        Ok(DocumentExtractFlightRouteResponse::from_batches(
            engine_batches,
        ))
    }

    async fn reuse_succeeded_artifact(
        &self,
        source: &Path,
        output: &Path,
    ) -> Result<Option<DocumentExtractFlightRouteResponse>, String> {
        let status = {
            let _registry_guard = self.registry_lock();
            self.registry()?
                .succeeded_status_for_source_content(source)?
        };
        let Some(status) = status else {
            return Ok(None);
        };
        let _guard = self.runtime.artifact_lock.lock().await;
        Self::mirror_and_read_succeeded(&status, output).map(Some)
    }

    pub(super) async fn persist_sync_output_artifact(
        &self,
        source: &Path,
        output: &Path,
    ) -> Result<(), String> {
        let artifact_dir = {
            let _registry_guard = self.registry_lock();
            self.registry()?.artifact_dir_for_source_content(source)?
        };
        let _guard = self.runtime.artifact_lock.lock().await;
        if artifact_dir.exists() {
            std::fs::remove_dir_all(artifact_dir.as_path()).map_err(|error| {
                format!(
                    "remove stale sync document extract artifact `{}`: {error}",
                    artifact_dir.display()
                )
            })?;
        }
        mirror_document_extract_cache(output, artifact_dir.as_path())?;
        let _registry_guard = self.registry_lock();
        self.registry()?.record_succeeded_output(source, output)?;
        Ok(())
    }

    pub(crate) async fn submit_document_extract_job(
        &self,
        source_path: &str,
        output_dir: Option<&str>,
        force: bool,
        wait_ms: u64,
    ) -> Result<DocumentExtractJobStatus, String> {
        let source = PathBuf::from(source_path);
        let output = output_dir
            .filter(|value| !value.trim().is_empty())
            .map_or_else(|| default_output_dir(source.as_path()), PathBuf::from);
        let registry = self.registry()?;
        let status = {
            let _guard = self.runtime.submit_lock.lock().await;
            let _registry_guard = self.registry_lock();
            registry.submit(source.as_path(), output.as_path(), force)?
        };
        if matches!(status.status.as_str(), "queued" | "running") {
            self.schedule_job(status.job_id.clone()).await;
        }
        if wait_ms == 0 {
            return Ok(status);
        }
        self.wait_for_terminal_status(status, wait_ms).await
    }

    pub(super) async fn async_document_extract_batch(
        &self,
        request: &DocumentExtractFlightRequest,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let source = PathBuf::from(request.source_path.as_str());
        let output = if request.output_dir.trim().is_empty() {
            default_output_dir(source.as_path())
        } else {
            PathBuf::from(request.output_dir.as_str())
        };
        if source.exists()
            && !request.force
            && let Some(batches) = read_cached_document_batches(source.as_path(), output.as_path())?
        {
            return Ok(DocumentExtractFlightRouteResponse::from_batches(batches));
        }

        let output_string = output.to_string_lossy().to_string();
        let mut status = self
            .submit_document_extract_job(
                request.source_path.as_str(),
                Some(output_string.as_str()),
                request.force,
                request.wait_ms,
            )
            .await?;

        if status.status == "succeeded" {
            let _guard = self.runtime.artifact_lock.lock().await;
            Self::mirror_and_read_succeeded(&status, output.as_path())
        } else if status.status == "failed" {
            if request.error_row {
                Ok(DocumentExtractFlightRouteResponse::new(
                    build_error_resource_batch(&status)?,
                ))
            } else {
                Err(status.error_message)
            }
        } else {
            if request.wait_ms > 0
                && let Some(current) = self.status(status.job_id.as_str())?
            {
                status = current;
            }
            Ok(DocumentExtractFlightRouteResponse::new(
                build_job_resource_batch(&status)?,
            ))
        }
    }

    pub(super) async fn wait_for_terminal_status(
        &self,
        status: DocumentExtractJobStatus,
        wait_ms: u64,
    ) -> Result<DocumentExtractJobStatus, String> {
        let deadline = Instant::now() + Duration::from_millis(wait_ms);
        let mut current = status;
        loop {
            if matches!(current.status.as_str(), "succeeded" | "failed") {
                return Ok(current);
            }
            if Instant::now() >= deadline {
                return Ok(current);
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
            if let Some(next) = self.status(current.job_id.as_str())? {
                current = next;
            }
        }
    }

    pub(super) async fn schedule_job(&self, job_id: String) {
        let mut scheduled = self.runtime.scheduled.lock().await;
        if !scheduled.insert(job_id.clone()) {
            return;
        }
        drop(scheduled);

        let provider = self.clone();
        tokio::spawn(async move {
            if let Err(error) = provider.run_job(job_id.as_str()).await {
                log::warn!("document extract async job `{job_id}` failed: {error}");
            }
            provider
                .runtime
                .scheduled
                .lock()
                .await
                .remove(job_id.as_str());
        });
    }

    pub(super) async fn run_job(&self, job_id: &str) -> Result<(), String> {
        let _permit = Arc::clone(&self.runtime.conversion_permits)
            .acquire_owned()
            .await
            .map_err(|error| format!("acquire document extract conversion permit: {error}"))?;
        let Some(status) = ({
            let _registry_guard = self.registry_lock();
            self.registry()?.start_job(job_id)?
        }) else {
            return Ok(());
        };
        let artifact_dir = PathBuf::from(status.artifact_dir.as_str());
        if artifact_dir.exists() {
            tokio::fs::remove_dir_all(artifact_dir.as_path())
                .await
                .map_err(|error| {
                    format!(
                        "remove stale document extract artifact `{}`: {error}",
                        artifact_dir.display()
                    )
                })?;
        }
        tokio::fs::create_dir_all(artifact_dir.as_path())
            .await
            .map_err(|error| {
                format!(
                    "create document extract artifact `{}`: {error}",
                    artifact_dir.display()
                )
            })?;

        let conversion = self
            .request_python_document_extract(
                status.source_path.as_str(),
                status.artifact_dir.as_str(),
                true,
                false,
            )
            .await;

        match conversion {
            Ok(batches) => {
                let resources_path = artifact_dir.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME);
                if !resources_path.exists() {
                    write_arrow_file(resources_path.as_path(), &batches)?;
                    tokio::fs::File::create(artifact_dir.join("_complete.marker"))
                        .await
                        .map_err(|error| {
                            format!("touch document extract artifact marker: {error}")
                        })?;
                }
                mirror_artifact_to_output(
                    artifact_dir.as_path(),
                    Path::new(status.output_dir.as_str()),
                )?;
                let _registry_guard = self.registry_lock();
                self.registry()?.mark_succeeded(job_id)
            }
            Err(error) => {
                let _registry_guard = self.registry_lock();
                self.registry()?.mark_failed(job_id, error.as_str())?;
                Err(error)
            }
        }
    }

    fn mirror_and_read_succeeded(
        status: &DocumentExtractJobStatus,
        output_dir: &Path,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let artifact_dir = Path::new(status.artifact_dir.as_str());
        if artifact_ready(status) {
            mirror_artifact_to_output(artifact_dir, output_dir)?;
        }
        let resources_path = output_dir.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME);
        let batches = read_arrow_file(resources_path.as_path())?;
        if batches.is_empty() {
            return Err(format!(
                "document extract cache `{}` contained no batches",
                resources_path.display()
            ));
        }
        Ok(DocumentExtractFlightRouteResponse::from_batches(batches))
    }

    pub(super) fn registry(&self) -> Result<&DocumentExtractJobRegistry, String> {
        self.runtime
            .registry
            .as_ref()
            .as_ref()
            .map_err(Clone::clone)
    }

    pub(super) fn registry_lock(&self) -> std::sync::MutexGuard<'_, ()> {
        self.runtime
            .registry_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

pub(super) fn document_extract_batches_are_cacheable(batches: &[RecordBatch]) -> bool {
    !batches.is_empty() && batches.iter().all(document_extract_batch_is_cacheable)
}

fn document_extract_batch_is_cacheable(batch: &RecordBatch) -> bool {
    if batch.num_rows() == 0 {
        return false;
    }
    let Some(resource_types) = document_extract_string_column(batch, "resourceType") else {
        return false;
    };
    let Some(statuses) = document_extract_string_column(batch, "status") else {
        return false;
    };
    (0..batch.num_rows()).all(|row| {
        !resource_types.is_null(row)
            && resource_types.value(row) != "error"
            && !statuses.is_null(row)
            && statuses.value(row) == "ok"
    })
}

fn document_extract_string_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
) -> Option<&'a StringArray> {
    batch
        .column_by_name(name)?
        .as_any()
        .downcast_ref::<StringArray>()
}
