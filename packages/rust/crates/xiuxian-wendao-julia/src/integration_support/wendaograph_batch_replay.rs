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
use crate::integration_support::search_strategy_flow_candidates::SearchStrategyFlowCandidateInputBatch;
use crate::integration_support::search_strategy_flow_flight::{
    SearchStrategyFlowFlightMaterializationConfig, materialize_search_strategy_flow_routes,
    search_strategy_flow_candidate_input_batch_from_repo_search,
    search_strategy_flow_ontology_registry_tsv_from_semantic_scope,
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
/// Rust still owns candidate discovery, TSV construction, trace enrichment, and
/// materialization receipts; Julia remains the owner of strategy scoring and
/// frontier selection.
pub struct SearchStrategyFlowPersistentBatchHost {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    stderr: ChildStderr,
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
        for (intent, candidate_batch) in candidate_batches {
            write_payload(&mut self.stdin, intent)?;
            write_payload(&mut self.stdin, &candidate_batch.tsv)?;
            write_payload(&mut self.stdin, candidate_batch.source)?;
            write_payload(&mut self.stdin, &candidate_batch.discovery_receipt_json)?;
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
        branch_judgements_tsv: &str,
    ) -> Result<String, String> {
        self.submit_with_flight_materialization_and_branch_judgements_and_ontology_registry(
            intent,
            config,
            branch_judgements_tsv,
            "",
        )
        .await
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
        branch_judgements_tsv: &str,
        ontology_registry_tsv: &str,
    ) -> Result<String, String> {
        validate_search_strategy_flow_intent(intent)?;
        let candidate_batch =
            search_strategy_flow_candidate_input_batch_from_repo_search(intent, config).await?;
        let ontology_registry_tsv = if ontology_registry_tsv.is_empty() {
            search_strategy_flow_ontology_registry_tsv_from_semantic_scope(config).await?
        } else {
            ontology_registry_tsv.to_owned()
        };
        let mut traces = self.submit_with_branch_judgements_and_ontology_registry(vec![(
            intent,
            candidate_batch,
            branch_judgements_tsv,
            ontology_registry_tsv.as_str(),
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
    /// The method keeps the existing TSV, JSON trace, and Flight route
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

    fn submit_with_branch_judgements_and_ontology_registry(
        &mut self,
        candidate_batches: Vec<(&str, SearchStrategyFlowCandidateInputBatch, &str, &str)>,
    ) -> Result<Vec<String>, String> {
        let batch_count = validate_judged_batch_request(&candidate_batches)?;
        write_payload(&mut self.stdin, &batch_count.to_string())?;
        for (intent, candidate_batch, branch_judgements_tsv, ontology_registry_tsv) in
            candidate_batches
        {
            write_payload(&mut self.stdin, intent)?;
            write_payload(&mut self.stdin, &candidate_batch.tsv)?;
            write_payload(&mut self.stdin, candidate_batch.source)?;
            write_payload(&mut self.stdin, &candidate_batch.discovery_receipt_json)?;
            write_payload(&mut self.stdin, branch_judgements_tsv)?;
            write_payload(&mut self.stdin, ontology_registry_tsv)?;
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
    for (intent, candidate_batch) in candidate_batches {
        command
            .arg(intent)
            .arg(candidate_batch.tsv)
            .arg(candidate_batch.source)
            .arg(candidate_batch.discovery_receipt_json)
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
        debug_assert_eq!(
            candidate_batch.row_count,
            candidate_batch
                .tsv
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count()
        );
    }
    Ok(candidate_batches.len())
}

fn validate_judged_batch_request(
    candidate_batches: &[(&str, SearchStrategyFlowCandidateInputBatch, &str, &str)],
) -> Result<usize, String> {
    if candidate_batches.is_empty() {
        return Err(
            "SearchStrategyFlow batch request must include at least one candidate batch".to_owned(),
        );
    }
    for (intent, candidate_batch, _, _) in candidate_batches {
        validate_search_strategy_flow_intent(intent)?;
        debug_assert_eq!(
            candidate_batch.row_count,
            candidate_batch
                .tsv
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count()
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
