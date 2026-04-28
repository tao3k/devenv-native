use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex as StdMutex, OnceLock, Weak};
use std::time::{Duration, Instant};

use arrow::record_batch::RecordBatch as EngineRecordBatch;
use arrow_flight::FlightDescriptor;
use arrow_flight::client::FlightClient;
use arrow_flight::flight_service_client::FlightServiceClient as TonicFlightServiceClient;
use async_trait::async_trait;
use futures::TryStreamExt;
use tokio::sync::{Mutex, Semaphore};
use tonic::transport::{Channel, Endpoint};
use xiuxian_wendao_runtime::transport::{
    ANALYSIS_DOCUMENT_EXTRACT_ROUTE, DocumentExtractFlightRequest,
    DocumentExtractFlightRouteProvider, DocumentExtractFlightRouteResponse, DocumentExtractMode,
    WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER, WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER, WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER,
};

use super::arrow_cache::{
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME, build_error_resource_batch, build_job_resource_batch,
    build_status_batch, mirror_artifact_to_output, read_arrow_file, read_cached_document_batches,
    write_arrow_file,
};
use super::registry::{
    DocumentExtractJobRegistry, DocumentExtractJobRegistrySnapshot, DocumentExtractJobStatus,
    artifact_ready, default_output_dir,
};
use crate::gateway::studio::router::GatewayState;

const DEFAULT_DOCUMENT_EXTRACT_ENDPOINT: &str = "http://localhost:50051";
const DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES: usize = 256 * 1024 * 1024;
const DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS";
const DEFAULT_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS: usize = 4;

#[derive(Clone)]
pub(crate) struct StudioDocumentExtractFlightRouteProvider {
    runtime: Arc<DocumentExtractProviderRuntime>,
}

struct DocumentExtractProviderRuntime {
    channel: Arc<Mutex<Option<CachedDocumentExtractChannel>>>,
    registry: Arc<Result<DocumentExtractJobRegistry, String>>,
    registry_lock: Arc<StdMutex<()>>,
    scheduled: Arc<Mutex<HashSet<String>>>,
    submit_lock: Arc<Mutex<()>>,
    artifact_lock: Arc<Mutex<()>>,
    conversion_permits: Arc<Semaphore>,
    conversion_limit: usize,
}

#[derive(Clone)]
struct CachedDocumentExtractChannel {
    endpoint_url: String,
    channel: Channel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocumentExtractRuntimeSnapshot {
    pub(crate) max_running_conversions: usize,
    pub(crate) available_conversion_permits: usize,
    pub(crate) in_process_running_conversions: usize,
    pub(crate) in_process_scheduled_jobs: usize,
    pub(crate) registry: DocumentExtractJobRegistrySnapshot,
}

static DOCUMENT_EXTRACT_PROVIDER_RUNTIMES: OnceLock<
    StdMutex<HashMap<PathBuf, Weak<DocumentExtractProviderRuntime>>>,
> = OnceLock::new();

impl StudioDocumentExtractFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: &GatewayState) -> Self {
        Self {
            runtime: shared_document_extract_provider_runtime(state.studio.project_root.as_path()),
        }
    }

    #[cfg(test)]
    fn from_registry(
        registry: Result<DocumentExtractJobRegistry, String>,
        conversion_limit: usize,
    ) -> Self {
        Self {
            runtime: Arc::new(DocumentExtractProviderRuntime::new(
                registry,
                conversion_limit,
            )),
        }
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

    pub(crate) fn status(&self, job_id: &str) -> Result<Option<DocumentExtractJobStatus>, String> {
        let _registry_guard = self.registry_lock();
        self.registry()?.status(job_id)
    }

    pub(crate) async fn runtime_snapshot(&self) -> Result<DocumentExtractRuntimeSnapshot, String> {
        let scheduled_count = self.runtime.scheduled.lock().await.len();
        let available_conversion_permits = self.runtime.conversion_permits.available_permits();
        let registry_snapshot = {
            let _registry_guard = self.registry_lock();
            self.registry()?.snapshot()?
        };
        Ok(DocumentExtractRuntimeSnapshot {
            max_running_conversions: self.runtime.conversion_limit,
            available_conversion_permits,
            in_process_running_conversions: self
                .runtime
                .conversion_limit
                .saturating_sub(available_conversion_permits),
            in_process_scheduled_jobs: scheduled_count,
            registry: registry_snapshot,
        })
    }

    async fn channel_for_endpoint(&self, endpoint_url: &str) -> Result<Channel, String> {
        {
            let cached = self.runtime.channel.lock().await;
            if let Some(cached) = cached.as_ref()
                && cached.endpoint_url == endpoint_url
            {
                return Ok(cached.channel.clone());
            }
        }

        let endpoint = Endpoint::from_shared(endpoint_url.to_string()).map_err(|error| {
            format!("invalid document extract endpoint `{endpoint_url}`: {error}")
        })?;

        let channel = endpoint.connect().await.map_err(|error| {
            format!("failed to connect to document extract endpoint `{endpoint_url}`: {error}")
        })?;

        let mut cached = self.runtime.channel.lock().await;
        *cached = Some(CachedDocumentExtractChannel {
            endpoint_url: endpoint_url.to_string(),
            channel: channel.clone(),
        });
        Ok(channel)
    }

    async fn async_document_extract_batch(
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

    async fn wait_for_terminal_status(
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

    async fn schedule_job(&self, job_id: String) {
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

    async fn run_job(&self, job_id: &str) -> Result<(), String> {
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

    async fn request_python_document_extract(
        &self,
        source_path: &str,
        output_dir: &str,
        force: bool,
        error_row: bool,
    ) -> Result<Vec<EngineRecordBatch>, String> {
        let endpoint_url = std::env::var("WENDAO_DOCUMENT_EXTRACT_ENDPOINT")
            .unwrap_or_else(|_| DEFAULT_DOCUMENT_EXTRACT_ENDPOINT.to_string());

        let channel = self.channel_for_endpoint(&endpoint_url).await?;

        let inner_client = TonicFlightServiceClient::new(channel)
            .max_encoding_message_size(DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES)
            .max_decoding_message_size(DOCUMENT_EXTRACT_FLIGHT_MESSAGE_SIZE_BYTES);
        let mut client = FlightClient::new_from_inner(inner_client);
        client
            .add_header(WENDAO_SCHEMA_VERSION_HEADER, "v2")
            .map_err(|error| format!("invalid schema version header: {error}"))?;
        client
            .add_header(WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER, source_path)
            .map_err(|error| format!("invalid source path header: {error}"))?;
        client
            .add_header(WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER, output_dir)
            .map_err(|error| format!("invalid output dir header: {error}"))?;
        client
            .add_header(
                WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
                if force { "true" } else { "false" },
            )
            .map_err(|error| format!("invalid force header: {error}"))?;
        client
            .add_header(
                WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
                if error_row { "true" } else { "false" },
            )
            .map_err(|error| format!("invalid error-row header: {error}"))?;

        let descriptor = FlightDescriptor::new_path(
            ANALYSIS_DOCUMENT_EXTRACT_ROUTE
                .trim_start_matches('/')
                .split('/')
                .map(ToString::to_string)
                .collect(),
        );
        let flight_info = client
            .get_flight_info(descriptor)
            .await
            .map_err(|error| format!("document extract get_flight_info failed: {error}"))?;

        let ticket = flight_info
            .endpoint
            .first()
            .and_then(|endpoint| endpoint.ticket.clone())
            .ok_or_else(|| "document extract flight info missing ticket".to_string())?;

        let stream = client
            .do_get(ticket)
            .await
            .map_err(|error| format!("document extract do_get failed: {error}"))?;

        let engine_batches: Vec<EngineRecordBatch> = stream
            .try_collect()
            .await
            .map_err(|error| format!("document extract stream decode failed: {error}"))?;

        if engine_batches.is_empty() {
            return Err("document extract returned no record batches".to_string());
        }
        Ok(engine_batches)
    }

    fn registry(&self) -> Result<&DocumentExtractJobRegistry, String> {
        self.runtime
            .registry
            .as_ref()
            .as_ref()
            .map_err(Clone::clone)
    }

    fn registry_lock(&self) -> std::sync::MutexGuard<'_, ()> {
        self.runtime
            .registry_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl DocumentExtractProviderRuntime {
    fn new(registry: Result<DocumentExtractJobRegistry, String>, conversion_limit: usize) -> Self {
        let conversion_limit = conversion_limit.max(1);
        Self {
            channel: Arc::new(Mutex::new(None)),
            registry: Arc::new(registry),
            registry_lock: Arc::new(StdMutex::new(())),
            scheduled: Arc::new(Mutex::new(HashSet::new())),
            submit_lock: Arc::new(Mutex::new(())),
            artifact_lock: Arc::new(Mutex::new(())),
            conversion_permits: Arc::new(Semaphore::new(conversion_limit)),
            conversion_limit,
        }
    }
}

fn shared_document_extract_provider_runtime(
    project_root: &Path,
) -> Arc<DocumentExtractProviderRuntime> {
    let key = project_root.to_path_buf();
    let runtimes = DOCUMENT_EXTRACT_PROVIDER_RUNTIMES.get_or_init(|| StdMutex::new(HashMap::new()));
    let mut guard = runtimes
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(runtime) = guard.get(&key).and_then(Weak::upgrade) {
        return runtime;
    }

    let runtime = Arc::new(DocumentExtractProviderRuntime::new(
        DocumentExtractJobRegistry::default_for_project(project_root),
        document_extract_conversion_concurrency_limit(),
    ));
    guard.insert(key, Arc::downgrade(&runtime));
    runtime
}

fn document_extract_conversion_concurrency_limit() -> usize {
    document_extract_conversion_concurrency_limit_with_lookup(
        &|key| std::env::var(key).ok(),
        std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .ok(),
    )
}

fn document_extract_conversion_concurrency_limit_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
    available_parallelism: Option<usize>,
) -> usize {
    if let Some(limit) = lookup(DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS_ENV)
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|limit| *limit > 0)
    {
        return limit;
    }

    available_parallelism
        .unwrap_or(1)
        .clamp(1, DEFAULT_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS)
}

impl std::fmt::Debug for StudioDocumentExtractFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioDocumentExtractFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl DocumentExtractFlightRouteProvider for StudioDocumentExtractFlightRouteProvider {
    async fn document_extract_batch(
        &self,
        source_path: &str,
        output_dir: &str,
        force: bool,
        error_row: bool,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let engine_batches = self
            .request_python_document_extract(source_path, output_dir, force, error_row)
            .await?;
        Ok(DocumentExtractFlightRouteResponse::from_batches(
            engine_batches,
        ))
    }

    async fn document_extract_batch_for_request(
        &self,
        request: &DocumentExtractFlightRequest,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        match request.mode {
            DocumentExtractMode::Sync => {
                self.document_extract_batch(
                    request.source_path.as_str(),
                    request.output_dir.as_str(),
                    request.force,
                    request.error_row,
                )
                .await
            }
            DocumentExtractMode::Async => self.async_document_extract_batch(request).await,
        }
    }

    async fn document_extract_status_batch(
        &self,
        job_id: &str,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let status = self
            .status(job_id)?
            .ok_or_else(|| format!("unknown document extract job id `{job_id}`"))?;
        Ok(DocumentExtractFlightRouteResponse::new(build_status_batch(
            &status,
        )?))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tokio::time::{Duration, sleep};

    use super::*;

    #[test]
    fn document_extract_conversion_limit_defaults_to_bounded_parallelism() {
        let limit = document_extract_conversion_concurrency_limit_with_lookup(&|_| None, Some(12));

        assert_eq!(limit, DEFAULT_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS);
    }

    #[test]
    fn document_extract_conversion_limit_accepts_positive_override() {
        let limit = document_extract_conversion_concurrency_limit_with_lookup(
            &|key| (key == DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS_ENV).then(|| "7".to_string()),
            Some(12),
        );

        assert_eq!(limit, 7);
    }

    #[test]
    fn document_extract_conversion_limit_ignores_invalid_override() {
        let limit = document_extract_conversion_concurrency_limit_with_lookup(
            &|key| (key == DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS_ENV).then(|| "0".to_string()),
            Some(2),
        );

        assert_eq!(limit, 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn document_extract_job_remains_queued_until_conversion_permit_is_available()
    -> Result<(), String> {
        let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
        let source = temp.path().join("manual.pdf");
        fs::write(source.as_path(), b"pdf fixture").map_err(|error| error.to_string())?;
        let registry = DocumentExtractJobRegistry::new(
            temp.path().join("jobs.duckdb"),
            temp.path().join("artifacts"),
        )?;
        let provider = StudioDocumentExtractFlightRouteProvider::from_registry(Ok(registry), 1);
        let held_permit = Arc::clone(&provider.runtime.conversion_permits)
            .acquire_owned()
            .await
            .map_err(|error| error.to_string())?;
        let queued = provider.registry()?.submit(
            source.as_path(),
            temp.path().join("out").as_path(),
            false,
        )?;
        let job_id = queued.job_id.clone();
        let running_provider = provider.clone();

        let handle = tokio::spawn(async move { running_provider.run_job(job_id.as_str()).await });
        sleep(Duration::from_millis(50)).await;
        let status = provider
            .status(queued.job_id.as_str())?
            .ok_or_else(|| "job should still exist".to_string())?;

        assert_eq!(status.status, "queued");
        assert_eq!(status.attempt_count, 0);

        handle.abort();
        drop(held_permit);
        Ok(())
    }

    #[tokio::test]
    async fn document_extract_runtime_snapshot_reports_capacity_and_registry_counts()
    -> Result<(), String> {
        let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
        let source = temp.path().join("manual.pdf");
        fs::write(source.as_path(), b"pdf fixture").map_err(|error| error.to_string())?;
        let registry = DocumentExtractJobRegistry::new(
            temp.path().join("jobs.duckdb"),
            temp.path().join("artifacts"),
        )?;
        let provider = StudioDocumentExtractFlightRouteProvider::from_registry(Ok(registry), 2);
        let _held_permit = Arc::clone(&provider.runtime.conversion_permits)
            .acquire_owned()
            .await
            .map_err(|error| error.to_string())?;
        let queued = provider.registry()?.submit(
            source.as_path(),
            temp.path().join("out").as_path(),
            false,
        )?;
        provider.schedule_job(queued.job_id.clone()).await;

        let snapshot = provider.runtime_snapshot().await?;

        assert_eq!(snapshot.max_running_conversions, 2);
        assert_eq!(snapshot.available_conversion_permits, 1);
        assert_eq!(snapshot.in_process_running_conversions, 1);
        assert_eq!(snapshot.in_process_scheduled_jobs, 1);
        assert_eq!(snapshot.registry.queued_jobs, 1);
        assert_eq!(snapshot.registry.total_jobs, 1);
        Ok(())
    }

    #[test]
    fn document_extract_provider_reuses_runtime_for_same_project_root() {
        let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("create tempdir: {error}"));
        let first = shared_document_extract_provider_runtime(temp.path());
        let second = shared_document_extract_provider_runtime(temp.path());

        assert!(Arc::ptr_eq(&first, &second));
    }
}
