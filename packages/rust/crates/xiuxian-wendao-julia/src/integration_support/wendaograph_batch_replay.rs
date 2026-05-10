//! Batched `SearchStrategyFlow` host replay support.

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(test)]
use std::{
    io::{BufRead, BufReader, BufWriter, Read, Write},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Stdio},
};

use super::scripts::SEARCH_STRATEGY_FLOW_JULIA;
use super::{
    enrich_wendaograph_search_strategy_flow_retrieval_routes, resolve_existing_path,
    validate_search_strategy_flow_intent, wendaograph_julia_project,
};
use crate::integration_support::search_strategy_flow_candidates::SearchStrategyFlowCandidateInputBatch;

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

#[cfg(test)]
pub(crate) struct SearchStrategyFlowPersistentBatchHost {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    stderr: ChildStderr,
}

#[cfg(test)]
impl SearchStrategyFlowPersistentBatchHost {
    pub(crate) fn start(search_root: impl Into<PathBuf>) -> Result<Self, String> {
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

    pub(crate) fn finish(mut self) -> Result<(), String> {
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
            .arg(candidate_batch.source);
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

#[cfg(test)]
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

#[cfg(test)]
fn write_payload(writer: &mut BufWriter<ChildStdin>, value: &str) -> Result<(), String> {
    writer
        .write_all(format!("{}\n", value.len()).as_bytes())
        .and_then(|()| writer.write_all(value.as_bytes()))
        .map_err(|error| {
            format!("write persistent WendaoGraph SearchStrategyFlow batch payload: {error}")
        })
}

#[cfg(test)]
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
