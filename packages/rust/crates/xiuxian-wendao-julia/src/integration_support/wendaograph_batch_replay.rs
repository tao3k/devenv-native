//! Batched `SearchStrategyFlow` host replay support.

use std::{
    env,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio},
    time::Instant,
};

use super::persistent_host_report::{
    SearchStrategyFlowPersistentHostStabilizationLimits,
    SearchStrategyFlowPersistentHostStabilizationReason,
    SearchStrategyFlowPersistentHostStabilizationReport, warm_path_stats_from_samples,
};
use super::scripts::SEARCH_STRATEGY_FLOW_JULIA;
use super::{
    enrich_wendaograph_search_strategy_flow_retrieval_routes, resolve_existing_path,
    validate_search_strategy_flow_intent, wendaograph_julia_project,
};
use crate::integration_support::search_strategy_flow_candidates::{
    SearchStrategyFlowCandidateInputBatch, search_strategy_flow_candidate_input_batch_from_markdown,
};
use crate::integration_support::search_strategy_flow_flight::{
    SearchStrategyFlowArrowIpcFile, SearchStrategyFlowFlightMaterializationConfig,
    materialize_search_strategy_flow_routes,
    search_strategy_flow_candidate_input_batch_from_repo_search,
    search_strategy_flow_ontology_registry_arrow_ipc_from_semantic_scope,
};

/// Run a batched `WendaoGraph` `SearchStrategyFlow` JSON host replay.
///
/// # Errors
///
/// Returns an error when the request is empty, an intent is invalid, the Julia
/// host process fails, or the host returns a trace count that does not match
/// the submitted batch count.
pub fn run_wendaograph_search_strategy_flow_json_batch_with_candidate_batches(
    search_root: impl Into<PathBuf>,
    candidate_batches: Vec<(&str, SearchStrategyFlowCandidateInputBatch)>,
) -> Result<Vec<String>, String> {
    let traces = run_raw_json_batch_with_candidate_batches(search_root, candidate_batches)?;
    traces
        .iter()
        .map(|trace| enrich_wendaograph_search_strategy_flow_retrieval_routes(trace))
        .collect()
}

/// Persistent `WendaoGraph.jl` `SearchStrategyFlow` batch host.
///
/// The host keeps one Julia process warm across multiple batch submissions.
/// Rust still owns candidate discovery, Arrow IPC file staging, trace
/// enrichment, and materialization receipts; Julia remains the owner of
/// strategy scoring and frontier selection.
pub struct SearchStrategyFlowPersistentBatchHost {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    stderr: ChildStderr,
    ontology_registry_cache: Option<SearchStrategyFlowOntologyRegistryCache>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SearchStrategyFlowOntologyRegistryCacheKey {
    base_url: String,
    repo_id: String,
}

impl SearchStrategyFlowOntologyRegistryCacheKey {
    fn from_config(config: &SearchStrategyFlowFlightMaterializationConfig) -> Self {
        Self {
            base_url: config.base_url.clone(),
            repo_id: config.repo_id.clone(),
        }
    }
}

#[derive(Debug)]
struct SearchStrategyFlowOntologyRegistryCache {
    key: SearchStrategyFlowOntologyRegistryCacheKey,
    path: String,
    _file: SearchStrategyFlowArrowIpcFile,
}

pub(crate) struct SearchStrategyFlowSideTableRequest {
    pub(crate) intent: String,
    pub(crate) query_understanding_arrow_ipc_path: String,
    pub(crate) branch_judgements_arrow_ipc_path: String,
    pub(crate) ontology_registry_arrow_ipc_path: String,
}

impl SearchStrategyFlowSideTableRequest {
    pub(crate) fn new(
        intent: impl Into<String>,
        query_understanding_arrow_ipc_path: impl Into<String>,
        branch_judgements_arrow_ipc_path: impl Into<String>,
        ontology_registry_arrow_ipc_path: impl Into<String>,
    ) -> Self {
        Self {
            intent: intent.into(),
            query_understanding_arrow_ipc_path: query_understanding_arrow_ipc_path.into(),
            branch_judgements_arrow_ipc_path: branch_judgements_arrow_ipc_path.into(),
            ontology_registry_arrow_ipc_path: ontology_registry_arrow_ipc_path.into(),
        }
    }
}

struct SearchStrategyFlowSideTableBatchRequest {
    intent: String,
    candidate_batch: SearchStrategyFlowCandidateInputBatch,
    query_understanding_arrow_ipc_path: String,
    branch_judgements_arrow_ipc_path: String,
    ontology_registry_arrow_ipc_path: String,
}

impl SearchStrategyFlowPersistentBatchHost {
    /// Start a persistent Julia batch host for one resolved search root.
    ///
    /// # Errors
    ///
    /// Returns an error when the Julia project or search root cannot be
    /// resolved, the child process cannot be spawned, or the expected standard
    /// streams are not available.
    pub fn start(search_root: impl Into<PathBuf>) -> Result<Self, String> {
        let julia_project = wendaograph_julia_project()?;
        let search_root =
            resolve_existing_path("WendaoGraph SearchStrategyFlow search root", search_root)?;
        let mut child = search_strategy_flow_persistent_batch_command(&julia_project, &search_root)
            .spawn()
            .map_err(|error| {
                format!("spawn persistent WendaoGraph SearchStrategyFlow batch host: {error}")
            })?;
        let stdin = child.stdin.take().ok_or_else(|| {
            "persistent WendaoGraph SearchStrategyFlow batch host did not expose stdin".to_owned()
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            "persistent WendaoGraph SearchStrategyFlow batch host did not expose stdout".to_owned()
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            "persistent WendaoGraph SearchStrategyFlow batch host did not expose stderr".to_owned()
        })?;
        Ok(Self {
            child,
            stdin: BufWriter::new(stdin),
            stdout: BufReader::new(stdout),
            stderr,
            ontology_registry_cache: None,
        })
    }

    /// Submit one batch request to the warm Julia process.
    ///
    /// # Errors
    ///
    /// Returns an error when the batch request is invalid, stdin cannot be
    /// written, stdout cannot be read, or the trace count differs from the
    /// submitted batch count.
    #[cfg(test)]
    pub(crate) fn submit(
        &mut self,
        candidate_batches: Vec<(&str, SearchStrategyFlowCandidateInputBatch)>,
    ) -> Result<Vec<String>, String> {
        let batch_count = validate_batch_request(&candidate_batches)?;
        write_payload(&mut self.stdin, &batch_count.to_string())?;
        let mut candidate_files = Vec::with_capacity(candidate_batches.len());
        for (intent, candidate_batch) in candidate_batches {
            let candidate_file = SearchStrategyFlowArrowIpcFile::write(
                "strategy-candidates",
                &candidate_batch.candidate_input_arrow_ipc_stream,
            )?;
            let candidate_arrow_ipc_path = candidate_file.path().to_string_lossy().into_owned();
            candidate_files.push(candidate_file);
            write_payload(&mut self.stdin, intent)?;
            write_payload(&mut self.stdin, candidate_arrow_ipc_path.as_str())?;
            write_payload(&mut self.stdin, candidate_batch.source)?;
            write_payload(&mut self.stdin, &candidate_batch.discovery_receipt_json)?;
            write_payload(&mut self.stdin, "")?;
            write_payload(&mut self.stdin, "")?;
            write_payload(&mut self.stdin, "")?;
        }
        self.stdin.flush().map_err(|error| {
            format!("flush persistent WendaoGraph SearchStrategyFlow batch host stdin: {error}")
        })?;

        let traces = read_batch_stdout_lines(&mut self.stdout, batch_count)?;
        traces
            .iter()
            .map(|trace| enrich_wendaograph_search_strategy_flow_retrieval_routes(trace))
            .collect()
    }

    /// Discover candidates through Flight, submit them to the warm Julia host,
    /// and materialize the planned routes through the same Flight endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error when Flight candidate discovery fails, the warm Julia
    /// host returns an invalid trace count, JSON trace parsing fails, or Flight
    /// route materialization fails.
    pub async fn submit_with_flight_materialization(
        &mut self,
        intent: &str,
        config: &SearchStrategyFlowFlightMaterializationConfig,
    ) -> Result<String, String> {
        self.submit_with_flight_materialization_and_branch_judgements(intent, config, "")
            .await
    }

    /// Discover candidates through Flight, submit them with Agent branch
    /// judgements to the warm Julia process, and materialize planned routes.
    ///
    /// # Errors
    ///
    /// Returns an error when Flight candidate discovery fails, the warm Julia
    /// host returns an invalid trace count, JSON trace parsing fails, or Flight
    /// route materialization fails.
    pub async fn submit_with_flight_materialization_and_branch_judgements(
        &mut self,
        intent: &str,
        config: &SearchStrategyFlowFlightMaterializationConfig,
        branch_judgements_arrow_ipc_path: &str,
    ) -> Result<String, String> {
        self.submit_with_flight_materialization_and_branch_judgements_and_ontology_registry(
            intent,
            config,
            branch_judgements_arrow_ipc_path,
            "",
        )
        .await
    }

    /// Discover candidates through Flight, submit them with optional
    /// query-understanding, Agent branch judgement, and ontology registry rows
    /// to the warm Julia process, and materialize planned routes.
    ///
    /// # Errors
    ///
    /// Returns an error when Flight candidate discovery fails, the warm Julia
    /// host returns an invalid trace count, JSON trace parsing fails, or Flight
    /// route materialization fails.
    pub async fn submit_with_flight_materialization_and_side_tables(
        &mut self,
        intent: &str,
        config: &SearchStrategyFlowFlightMaterializationConfig,
        query_understanding_arrow_ipc_path: &str,
        branch_judgements_arrow_ipc_path: &str,
        ontology_registry_arrow_ipc_path: &str,
    ) -> Result<String, String> {
        let mut traces = self
            .submit_batch_with_flight_materialization_and_side_tables(
                vec![SearchStrategyFlowSideTableRequest::new(
                    intent,
                    query_understanding_arrow_ipc_path,
                    branch_judgements_arrow_ipc_path,
                    ontology_registry_arrow_ipc_path,
                )],
                config,
            )
            .await?;
        traces.pop().ok_or_else(|| {
            "persistent WendaoGraph SearchStrategyFlow host returned no trace".to_owned()
        })
    }

    /// Discover multiple candidate batches through Flight, submit them to the
    /// warm Julia process together, and materialize each planned route.
    ///
    /// # Errors
    ///
    /// Returns an error when candidate discovery, Julia batch submit, JSON
    /// parsing, or Flight route materialization fails for any request.
    pub(crate) async fn submit_batch_with_flight_materialization_and_side_tables(
        &mut self,
        requests: Vec<SearchStrategyFlowSideTableRequest>,
        config: &SearchStrategyFlowFlightMaterializationConfig,
    ) -> Result<Vec<String>, String> {
        let mut batch_requests = Vec::with_capacity(requests.len());
        for request in requests {
            validate_search_strategy_flow_intent(&request.intent)?;
            let candidate_batch = search_strategy_flow_candidate_input_batch_from_repo_search(
                &request.intent,
                config,
            )
            .await?;
            let ontology_registry_arrow_ipc_path = self
                .ontology_registry_arrow_ipc_path(
                    config,
                    request.ontology_registry_arrow_ipc_path.as_str(),
                )
                .await?;
            batch_requests.push(SearchStrategyFlowSideTableBatchRequest {
                intent: request.intent,
                candidate_batch,
                query_understanding_arrow_ipc_path: request.query_understanding_arrow_ipc_path,
                branch_judgements_arrow_ipc_path: request.branch_judgements_arrow_ipc_path,
                ontology_registry_arrow_ipc_path,
            });
        }
        let traces = self.submit_owned_side_tables(batch_requests)?;
        let mut materialized_traces = Vec::with_capacity(traces.len());
        for trace in traces {
            let mut value = serde_json::from_str::<serde_json::Value>(&trace).map_err(|error| {
                format!("parse persistent WendaoGraph SearchStrategyFlow JSON trace: {error}")
            })?;
            materialize_search_strategy_flow_routes(&mut value, config).await?;
            materialized_traces.push(
                serde_json::to_string(&value)
                    .map(|trace| format!("{trace}\n"))
                    .map_err(|error| {
                        format!(
                            "serialize persistent SearchStrategyFlow materialized trace: {error}"
                        )
                    })?,
            );
        }
        Ok(materialized_traces)
    }

    /// Discover candidates from local Markdown, submit them with optional side
    /// table paths to the warm Julia process, and return planned route receipts.
    ///
    /// # Errors
    ///
    /// Returns an error when local candidate discovery fails, the warm Julia
    /// host returns an invalid trace count, or JSON trace enrichment fails.
    pub fn submit_with_markdown_candidates_and_side_tables(
        &mut self,
        intent: &str,
        search_root: impl AsRef<Path>,
        query_understanding_arrow_ipc_path: &str,
        branch_judgements_arrow_ipc_path: &str,
        ontology_registry_arrow_ipc_path: &str,
    ) -> Result<String, String> {
        let mut traces = self.submit_batch_with_markdown_candidates_and_side_tables(
            search_root,
            vec![SearchStrategyFlowSideTableRequest::new(
                intent,
                query_understanding_arrow_ipc_path,
                branch_judgements_arrow_ipc_path,
                ontology_registry_arrow_ipc_path,
            )],
        )?;
        traces.pop().ok_or_else(|| {
            "persistent WendaoGraph SearchStrategyFlow host returned no trace".to_owned()
        })
    }

    /// Discover local Markdown candidates for multiple requests and submit one
    /// Julia batch to amortize host-side strategy selection overhead.
    ///
    /// # Errors
    ///
    /// Returns an error when local candidate discovery or the Julia batch host
    /// fails.
    pub(crate) fn submit_batch_with_markdown_candidates_and_side_tables(
        &mut self,
        search_root: impl AsRef<Path>,
        requests: Vec<SearchStrategyFlowSideTableRequest>,
    ) -> Result<Vec<String>, String> {
        let mut batch_requests = Vec::with_capacity(requests.len());
        for request in requests {
            validate_search_strategy_flow_intent(&request.intent)?;
            let candidate_batch = search_strategy_flow_candidate_input_batch_from_markdown(
                &request.intent,
                search_root.as_ref(),
            )?;
            batch_requests.push(SearchStrategyFlowSideTableBatchRequest {
                intent: request.intent,
                candidate_batch,
                query_understanding_arrow_ipc_path: request.query_understanding_arrow_ipc_path,
                branch_judgements_arrow_ipc_path: request.branch_judgements_arrow_ipc_path,
                ontology_registry_arrow_ipc_path: request.ontology_registry_arrow_ipc_path,
            });
        }
        self.submit_owned_side_tables(batch_requests)
    }

    /// Discover candidates through Flight, submit them with optional Agent
    /// branch judgements and ontology registry rows to the warm Julia process,
    /// and materialize planned routes.
    ///
    /// # Errors
    ///
    /// Returns an error when Flight candidate discovery fails, the warm Julia
    /// host returns an invalid trace count, JSON trace parsing fails, or Flight
    /// route materialization fails.
    pub async fn submit_with_flight_materialization_and_branch_judgements_and_ontology_registry(
        &mut self,
        intent: &str,
        config: &SearchStrategyFlowFlightMaterializationConfig,
        branch_judgements_arrow_ipc_path: &str,
        ontology_registry_arrow_ipc_path: &str,
    ) -> Result<String, String> {
        self.submit_with_flight_materialization_and_query_understanding_branch_judgements_and_ontology_registry(
            intent,
            config,
            "",
            branch_judgements_arrow_ipc_path,
            ontology_registry_arrow_ipc_path,
        )
        .await
    }

    async fn submit_with_flight_materialization_and_query_understanding_branch_judgements_and_ontology_registry(
        &mut self,
        intent: &str,
        config: &SearchStrategyFlowFlightMaterializationConfig,
        query_understanding_arrow_ipc_path: &str,
        branch_judgements_arrow_ipc_path: &str,
        ontology_registry_arrow_ipc_path: &str,
    ) -> Result<String, String> {
        validate_search_strategy_flow_intent(intent)?;
        let candidate_batch =
            search_strategy_flow_candidate_input_batch_from_repo_search(intent, config).await?;
        let ontology_registry_path = self
            .ontology_registry_arrow_ipc_path(config, ontology_registry_arrow_ipc_path)
            .await?;
        let mut traces = self.submit_with_side_tables(vec![(
            intent,
            candidate_batch,
            query_understanding_arrow_ipc_path,
            branch_judgements_arrow_ipc_path,
            ontology_registry_path.as_str(),
        )])?;
        let trace = traces.pop().ok_or_else(|| {
            "persistent WendaoGraph SearchStrategyFlow host returned no trace".to_owned()
        })?;
        let mut value = serde_json::from_str::<serde_json::Value>(&trace).map_err(|error| {
            format!("parse persistent WendaoGraph SearchStrategyFlow JSON trace: {error}")
        })?;
        materialize_search_strategy_flow_routes(&mut value, config).await?;
        serde_json::to_string(&value)
            .map(|trace| format!("{trace}\n"))
            .map_err(|error| {
                format!("serialize persistent SearchStrategyFlow materialized trace: {error}")
            })
    }

    /// Prewarm and sample this host before releasing it to user-visible
    /// `SearchStrategyFlow` traffic.
    ///
    /// The method keeps the existing Arrow IPC, JSON trace, and Flight route
    /// contracts intact. It only turns warm-host measurements into a Rust
    /// admission recommendation.
    ///
    /// # Errors
    ///
    /// Returns an error when the prewarm submit or any warm sample fails.
    pub async fn stabilize_with_flight_materialization(
        &mut self,
        intent: &str,
        config: &SearchStrategyFlowFlightMaterializationConfig,
        limits: SearchStrategyFlowPersistentHostStabilizationLimits,
    ) -> Result<SearchStrategyFlowPersistentHostStabilizationReport, String> {
        validate_search_strategy_flow_intent(intent)?;
        let started = Instant::now();
        self.submit_with_flight_materialization(intent, config)
            .await?;
        let prewarm_elapsed = started.elapsed();

        let mut samples = Vec::with_capacity(limits.sample_count);
        for _ in 0..limits.sample_count {
            let sample_started = Instant::now();
            self.submit_with_flight_materialization(intent, config)
                .await?;
            samples.push(sample_started.elapsed());
        }

        let warm = warm_path_stats_from_samples(&samples);
        let stability_reason = limits.stability_reason_for(&warm);
        let stable =
            stability_reason == SearchStrategyFlowPersistentHostStabilizationReason::Stable;
        let recommended_max_in_flight = limits.recommended_max_in_flight_for(stability_reason);

        Ok(SearchStrategyFlowPersistentHostStabilizationReport {
            prewarm_elapsed,
            warm,
            stable,
            stability_reason,
            recommended_max_in_flight,
        })
    }

    fn submit_with_side_tables(
        &mut self,
        candidate_batches: Vec<(
            &str,
            SearchStrategyFlowCandidateInputBatch,
            &str,
            &str,
            &str,
        )>,
    ) -> Result<Vec<String>, String> {
        let batch_requests = candidate_batches
            .into_iter()
            .map(
                |(
                    intent,
                    candidate_batch,
                    query_understanding_arrow_ipc_path,
                    branch_judgements_arrow_ipc_path,
                    ontology_registry_arrow_ipc_path,
                )| SearchStrategyFlowSideTableBatchRequest {
                    intent: intent.to_owned(),
                    candidate_batch,
                    query_understanding_arrow_ipc_path: query_understanding_arrow_ipc_path
                        .to_owned(),
                    branch_judgements_arrow_ipc_path: branch_judgements_arrow_ipc_path.to_owned(),
                    ontology_registry_arrow_ipc_path: ontology_registry_arrow_ipc_path.to_owned(),
                },
            )
            .collect();
        self.submit_owned_side_tables(batch_requests)
    }

    fn submit_owned_side_tables(
        &mut self,
        candidate_batches: Vec<SearchStrategyFlowSideTableBatchRequest>,
    ) -> Result<Vec<String>, String> {
        let batch_count = validate_side_table_batch_request(&candidate_batches)?;
        write_payload(&mut self.stdin, &batch_count.to_string())?;
        let mut candidate_files = Vec::with_capacity(candidate_batches.len());
        for request in candidate_batches {
            let candidate_file = SearchStrategyFlowArrowIpcFile::write(
                "strategy-candidates",
                &request.candidate_batch.candidate_input_arrow_ipc_stream,
            )?;
            let candidate_arrow_ipc_path = candidate_file.path().to_string_lossy().into_owned();
            candidate_files.push(candidate_file);
            write_payload(&mut self.stdin, request.intent.as_str())?;
            write_payload(&mut self.stdin, candidate_arrow_ipc_path.as_str())?;
            write_payload(&mut self.stdin, request.candidate_batch.source)?;
            write_payload(
                &mut self.stdin,
                &request.candidate_batch.discovery_receipt_json,
            )?;
            write_payload(
                &mut self.stdin,
                request.branch_judgements_arrow_ipc_path.as_str(),
            )?;
            write_payload(
                &mut self.stdin,
                request.ontology_registry_arrow_ipc_path.as_str(),
            )?;
            write_payload(
                &mut self.stdin,
                request.query_understanding_arrow_ipc_path.as_str(),
            )?;
        }
        self.stdin.flush().map_err(|error| {
            format!("flush persistent WendaoGraph SearchStrategyFlow batch host stdin: {error}")
        })?;

        let traces = read_batch_stdout_lines(&mut self.stdout, batch_count)?;
        traces
            .iter()
            .map(|trace| enrich_wendaograph_search_strategy_flow_retrieval_routes(trace))
            .collect()
    }

    async fn ontology_registry_arrow_ipc_path(
        &mut self,
        config: &SearchStrategyFlowFlightMaterializationConfig,
        explicit_path: &str,
    ) -> Result<String, String> {
        if !explicit_path.is_empty() {
            return Ok(explicit_path.to_owned());
        }
        let key = SearchStrategyFlowOntologyRegistryCacheKey::from_config(config);
        if let Some(cache) = self.ontology_registry_cache.as_ref()
            && cache.key == key
        {
            return Ok(cache.path.clone());
        }

        let payload =
            search_strategy_flow_ontology_registry_arrow_ipc_from_semantic_scope(config).await?;
        let file = SearchStrategyFlowArrowIpcFile::write("ontology-registry", &payload)?;
        let path = file.path().to_string_lossy().into_owned();
        self.ontology_registry_cache = Some(SearchStrategyFlowOntologyRegistryCache {
            key,
            path: path.clone(),
            _file: file,
        });
        Ok(path)
    }

    /// Finish the persistent Julia host and surface any process failure.
    ///
    /// # Errors
    ///
    /// Returns an error when waiting for the child process or reading stderr
    /// fails, or when the child process exits unsuccessfully.
    pub fn finish(mut self) -> Result<(), String> {
        drop(self.stdin);
        drop(self.stdout);
        let status = self.child.wait().map_err(|error| {
            format!("wait for persistent WendaoGraph SearchStrategyFlow batch host: {error}")
        })?;
        let mut stderr = String::new();
        self.stderr.read_to_string(&mut stderr).map_err(|error| {
            format!("read persistent WendaoGraph SearchStrategyFlow batch host stderr: {error}")
        })?;
        if !status.success() {
            return Err(format!(
                "persistent WendaoGraph SearchStrategyFlow batch host exited with status {status}; stderr:\n{stderr}"
            ));
        }
        Ok(())
    }
}

fn run_raw_json_batch_with_candidate_batches(
    search_root: impl Into<PathBuf>,
    candidate_batches: Vec<(&str, SearchStrategyFlowCandidateInputBatch)>,
) -> Result<Vec<String>, String> {
    let batch_count = validate_batch_request(&candidate_batches)?;
    let julia_project = wendaograph_julia_project()?;
    let search_root =
        resolve_existing_path("WendaoGraph SearchStrategyFlow search root", search_root)?;
    let mut command = search_strategy_flow_batch_command(&julia_project, &search_root, batch_count);
    let mut candidate_files = Vec::with_capacity(candidate_batches.len());
    for (intent, candidate_batch) in candidate_batches {
        let candidate_file = SearchStrategyFlowArrowIpcFile::write(
            "strategy-candidates",
            &candidate_batch.candidate_input_arrow_ipc_stream,
        )?;
        let candidate_arrow_ipc_path = candidate_file.path().to_string_lossy().into_owned();
        candidate_files.push(candidate_file);
        command
            .arg(intent)
            .arg(candidate_arrow_ipc_path)
            .arg(candidate_batch.source)
            .arg(candidate_batch.discovery_receipt_json)
            .arg("")
            .arg("")
            .arg("");
    }

    let output = command.output().map_err(|error| {
        format!("spawn WendaoGraph SearchStrategyFlow batch host request: {error}")
    })?;
    if !output.status.success() {
        let status = output.status;
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "WendaoGraph SearchStrategyFlow batch host request exited with status {status}; stderr:\n{stderr}"
        ));
    }

    parse_batch_stdout(&output.stdout, batch_count)
}

fn validate_batch_request(
    candidate_batches: &[(&str, SearchStrategyFlowCandidateInputBatch)],
) -> Result<usize, String> {
    if candidate_batches.is_empty() {
        return Err(
            "SearchStrategyFlow batch request must include at least one candidate batch".to_owned(),
        );
    }
    for (intent, candidate_batch) in candidate_batches {
        validate_search_strategy_flow_intent(intent)?;
        debug_assert!(
            candidate_batch.row_count == 0
                || !candidate_batch.candidate_input_arrow_ipc_stream.is_empty()
        );
    }
    Ok(candidate_batches.len())
}

fn validate_side_table_batch_request(
    candidate_batches: &[SearchStrategyFlowSideTableBatchRequest],
) -> Result<usize, String> {
    if candidate_batches.is_empty() {
        return Err(
            "SearchStrategyFlow batch request must include at least one candidate batch".to_owned(),
        );
    }
    for request in candidate_batches {
        validate_search_strategy_flow_intent(&request.intent)?;
        debug_assert!(
            request.candidate_batch.row_count == 0
                || !request
                    .candidate_batch
                    .candidate_input_arrow_ipc_stream
                    .is_empty()
        );
    }
    Ok(candidate_batches.len())
}

fn search_strategy_flow_batch_command(
    julia_project: &Path,
    search_root: &Path,
    batch_count: usize,
) -> Command {
    let julia_command = env::var("JULIA").unwrap_or_else(|_| "julia".to_owned());
    let mut command = Command::new(julia_command);
    command
        .arg(format!("--project={}", julia_project.display()))
        .arg("--startup-file=no")
        .arg("-e")
        .arg(SEARCH_STRATEGY_FLOW_JULIA)
        .arg("__WENDAO_SEARCH_STRATEGY_FLOW_BATCH__")
        .arg(search_root)
        .arg(batch_count.to_string());
    command
}

fn search_strategy_flow_persistent_batch_command(
    julia_project: &Path,
    search_root: &Path,
) -> Command {
    let julia_command = env::var("JULIA").unwrap_or_else(|_| "julia".to_owned());
    let mut command = Command::new(julia_command);
    command
        .arg(format!("--project={}", julia_project.display()))
        .arg("--startup-file=no")
        .arg("-e")
        .arg(SEARCH_STRATEGY_FLOW_JULIA)
        .arg("__WENDAO_SEARCH_STRATEGY_FLOW_BATCH_STDIN__")
        .arg(search_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

fn parse_batch_stdout(stdout: &[u8], batch_count: usize) -> Result<Vec<String>, String> {
    let stdout = String::from_utf8_lossy(stdout);
    let traces = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if traces.is_empty() {
        return Err(
            "WendaoGraph SearchStrategyFlow batch host request returned empty output".to_owned(),
        );
    }
    if traces.len() != batch_count {
        return Err(format!(
            "WendaoGraph SearchStrategyFlow batch host request returned {} traces for {batch_count} batches",
            traces.len()
        ));
    }
    Ok(traces)
}

fn write_payload(writer: &mut BufWriter<ChildStdin>, value: &str) -> Result<(), String> {
    writer
        .write_all(format!("{}\n", value.len()).as_bytes())
        .and_then(|()| writer.write_all(value.as_bytes()))
        .map_err(|error| {
            format!("write persistent WendaoGraph SearchStrategyFlow batch payload: {error}")
        })
}

fn read_batch_stdout_lines(
    reader: &mut BufReader<ChildStdout>,
    batch_count: usize,
) -> Result<Vec<String>, String> {
    let mut traces = Vec::with_capacity(batch_count);
    for index in 0..batch_count {
        let mut line = String::new();
        let byte_count = reader.read_line(&mut line).map_err(|error| {
            format!("read persistent WendaoGraph SearchStrategyFlow trace {index}: {error}")
        })?;
        if byte_count == 0 {
            return Err(format!(
                "persistent WendaoGraph SearchStrategyFlow batch host closed stdout after {index} traces; expected {batch_count}"
            ));
        }
        traces.push(line.trim().to_owned());
    }
    Ok(traces)
}

#[cfg(test)]
#[path = "../../tests/unit/integration_support/wendaograph_batch_replay.rs"]
mod tests;
