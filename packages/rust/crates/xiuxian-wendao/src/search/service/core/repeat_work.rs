use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::types::SearchPlaneService;
use crate::search::contracts::{ProjectConfigView, materialize_project_configs};
use crate::search::{ProjectScanInventory, ProjectScannedFile, scan_supported_project_files};
#[cfg(any(test, feature = "test-support"))]
use crate::search::{
    fingerprint_note_projects_with_scanned_files, fingerprint_source_projects_with_scanned_files,
    fingerprint_symbol_projects_with_scanned_files,
};

const MAX_TELEMETRY_PATH_SAMPLES: usize = 10;
const MAX_TELEMETRY_HOT_PATHS: usize = 20;
const MAX_TELEMETRY_FINDINGS: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchBuildRepeatWorkSummaryTelemetry {
    #[serde(rename = "totalFileObservationCount")]
    pub file_observations_total: u64,
    #[serde(rename = "totalUniquePathCount")]
    pub unique_paths_total: usize,
    #[serde(rename = "repeatedFileObservationCount")]
    pub repeated_file_observations: u64,
    #[serde(rename = "sourceOperationCount")]
    pub source_operations: usize,
    #[serde(rename = "hotPathCount")]
    pub hot_paths: usize,
    #[serde(rename = "findingCount")]
    pub findings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchBuildRepeatWorkPathTelemetry {
    pub path: String,
    pub observations: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchBuildRepeatWorkSourceTelemetry {
    pub source: String,
    pub operation: String,
    pub batch_count: u64,
    pub file_observation_count: u64,
    pub unique_path_count: usize,
    pub repeated_path_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_repeated_paths: Vec<SearchBuildRepeatWorkPathTelemetry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchBuildRepeatWorkHotPathTelemetry {
    pub path: String,
    pub observations: u64,
    pub source_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchBuildRepeatWorkFindingTelemetry {
    pub kind: String,
    pub severity: String,
    pub observations: u64,
    pub repeated_observations: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unique_path_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeated_path_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
/// Aggregated repeat-work telemetry for search index build scans.
pub struct SearchBuildRepeatWorkTelemetry {
    /// Summary counters across all observed source operations.
    pub summary: SearchBuildRepeatWorkSummaryTelemetry,
    /// Per-source operation telemetry.
    pub source_operations: Vec<SearchBuildRepeatWorkSourceTelemetry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Paths observed repeatedly across source operations.
    pub hot_paths: Vec<SearchBuildRepeatWorkHotPathTelemetry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Derived repeat-work findings sorted by severity and impact.
    pub findings: Vec<SearchBuildRepeatWorkFindingTelemetry>,
}

#[derive(Debug, Default)]
pub(crate) struct SearchBuildRepeatWorkTelemetryState {
    source_operations: BTreeMap<(String, String), SearchBuildRepeatWorkSourceState>,
}

#[derive(Debug, Default)]
struct SearchBuildRepeatWorkSourceState {
    batch_count: u64,
    file_observation_count: u64,
    path_counts: BTreeMap<String, u64>,
}

#[derive(Debug, Default)]
struct SearchBuildRepeatWorkHotPathState {
    observations: u64,
    sources: BTreeSet<String>,
    operations: BTreeSet<String>,
}

impl SearchPlaneService {
    pub(crate) fn record_repeat_work_paths<'a>(
        &self,
        source: &'static str,
        operation: &'static str,
        paths: impl IntoIterator<Item = &'a str>,
    ) {
        let mut telemetry = self
            .repeat_work_telemetry
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let entry = telemetry
            .source_operations
            .entry((source.to_string(), operation.to_string()))
            .or_default();
        entry.batch_count = entry.batch_count.saturating_add(1);
        for path in paths {
            entry.file_observation_count = entry.file_observation_count.saturating_add(1);
            *entry.path_counts.entry(path.to_string()).or_default() += 1;
        }
    }

    pub(crate) fn record_repeat_work_file(
        &self,
        source: &'static str,
        operation: &'static str,
        path: &str,
    ) {
        self.record_repeat_work_paths(source, operation, std::iter::once(path));
    }

    pub(crate) fn record_repeat_work_scanned_files(
        &self,
        source: &'static str,
        operation: &'static str,
        files: &[ProjectScannedFile],
    ) {
        self.record_repeat_work_paths(
            source,
            operation,
            files.iter().map(|file| file.normalized_path.as_str()),
        );
    }

    #[must_use]
    /// Return a compact repeat-work telemetry snapshot for diagnostics.
    pub fn repeat_work_telemetry(&self) -> SearchBuildRepeatWorkTelemetry {
        let telemetry = self
            .repeat_work_telemetry
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let total_file_observation_count = telemetry
            .source_operations
            .values()
            .map(|state| state.file_observation_count)
            .sum::<u64>();
        let total_unique_path_count = telemetry
            .source_operations
            .values()
            .flat_map(|state| state.path_counts.keys().cloned())
            .collect::<BTreeSet<_>>()
            .len();
        let mut hot_paths = BTreeMap::<String, SearchBuildRepeatWorkHotPathState>::new();
        let source_operations = telemetry
            .source_operations
            .iter()
            .map(|((source, operation), state)| {
                for (path, observations) in &state.path_counts {
                    let hot_path = hot_paths.entry(path.clone()).or_default();
                    hot_path.observations = hot_path.observations.saturating_add(*observations);
                    hot_path.sources.insert(source.clone());
                    hot_path.operations.insert(operation.clone());
                }
                SearchBuildRepeatWorkSourceTelemetry {
                    source: source.clone(),
                    operation: operation.clone(),
                    batch_count: state.batch_count,
                    file_observation_count: state.file_observation_count,
                    unique_path_count: state.path_counts.len(),
                    repeated_path_count: state
                        .path_counts
                        .values()
                        .filter(|count| **count > 1)
                        .count(),
                    top_repeated_paths: top_repeated_paths(&state.path_counts),
                }
            })
            .collect::<Vec<_>>();
        let source_operations = sort_source_operations(source_operations);
        let hot_paths = top_hot_paths(hot_paths);
        let findings = top_findings(build_findings(
            source_operations.as_slice(),
            hot_paths.as_slice(),
        ));
        let summary = build_summary(
            total_file_observation_count,
            total_unique_path_count,
            source_operations.as_slice(),
            hot_paths.as_slice(),
            findings.as_slice(),
        );

        SearchBuildRepeatWorkTelemetry {
            summary,
            source_operations,
            hot_paths,
            findings,
        }
    }

    #[must_use]
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn fingerprint_note_projects_with_repeat_work_details(
        &self,
        source: &'static str,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[impl ProjectConfigView],
    ) -> (String, Vec<ProjectScannedFile>) {
        let projects = materialize_project_configs(projects);
        let (fingerprint, files) = fingerprint_note_projects_with_scanned_files(
            project_root,
            config_root,
            projects.as_slice(),
        );
        self.record_repeat_work_scanned_files(source, "scan_note_project_files", &files);
        (fingerprint, files)
    }

    #[must_use]
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn fingerprint_source_projects_with_repeat_work_details(
        &self,
        source: &'static str,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[impl ProjectConfigView],
    ) -> (String, Vec<ProjectScannedFile>) {
        let projects = materialize_project_configs(projects);
        let (fingerprint, files) = fingerprint_source_projects_with_scanned_files(
            project_root,
            config_root,
            projects.as_slice(),
        );
        self.record_repeat_work_scanned_files(source, "scan_source_project_files", &files);
        (fingerprint, files)
    }

    #[must_use]
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn fingerprint_symbol_projects_with_repeat_work_details(
        &self,
        source: &'static str,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[impl ProjectConfigView],
    ) -> (String, Vec<ProjectScannedFile>) {
        let projects = materialize_project_configs(projects);
        let (fingerprint, files) = fingerprint_symbol_projects_with_scanned_files(
            project_root,
            config_root,
            projects.as_slice(),
        );
        self.record_repeat_work_scanned_files(source, "scan_symbol_project_files", &files);
        (fingerprint, files)
    }

    /// Scan supported project files and record repeat-work telemetry.
    #[must_use]
    pub fn scan_supported_projects_with_repeat_work_details(
        &self,
        source: &'static str,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[impl ProjectConfigView],
    ) -> ProjectScanInventory {
        let projects = materialize_project_configs(projects);
        let inventory =
            scan_supported_project_files(project_root, config_root, projects.as_slice());
        self.record_repeat_work_scanned_files(
            source,
            "scan_supported_project_files",
            inventory.symbol_files(),
        );
        inventory
    }
}

fn top_repeated_paths(
    path_counts: &BTreeMap<String, u64>,
) -> Vec<SearchBuildRepeatWorkPathTelemetry> {
    let mut repeated = path_counts
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|(path, observations)| SearchBuildRepeatWorkPathTelemetry {
            path: path.clone(),
            observations: *observations,
        })
        .collect::<Vec<_>>();
    repeated.sort_by(|left, right| {
        right
            .observations
            .cmp(&left.observations)
            .then_with(|| left.path.cmp(&right.path))
    });
    repeated.truncate(MAX_TELEMETRY_PATH_SAMPLES);
    repeated
}

fn sort_source_operations(
    mut observations: Vec<SearchBuildRepeatWorkSourceTelemetry>,
) -> Vec<SearchBuildRepeatWorkSourceTelemetry> {
    observations.sort_by(|left, right| {
        right
            .file_observation_count
            .cmp(&left.file_observation_count)
            .then_with(|| right.batch_count.cmp(&left.batch_count))
            .then_with(|| left.source.cmp(&right.source))
            .then_with(|| left.operation.cmp(&right.operation))
    });
    observations
}

fn top_hot_paths(
    hot_paths: BTreeMap<String, SearchBuildRepeatWorkHotPathState>,
) -> Vec<SearchBuildRepeatWorkHotPathTelemetry> {
    let mut telemetry = hot_paths
        .into_iter()
        .filter(|(_, state)| state.observations > 1)
        .map(|(path, state)| SearchBuildRepeatWorkHotPathTelemetry {
            path,
            observations: state.observations,
            source_count: state.sources.len(),
            sources: state.sources.into_iter().collect(),
            operations: state.operations.into_iter().collect(),
        })
        .collect::<Vec<_>>();
    telemetry.sort_by(|left, right| {
        right
            .observations
            .cmp(&left.observations)
            .then_with(|| right.source_count.cmp(&left.source_count))
            .then_with(|| left.path.cmp(&right.path))
    });
    telemetry.truncate(MAX_TELEMETRY_HOT_PATHS);
    telemetry
}

fn build_findings(
    source_operations: &[SearchBuildRepeatWorkSourceTelemetry],
    hot_paths: &[SearchBuildRepeatWorkHotPathTelemetry],
) -> Vec<SearchBuildRepeatWorkFindingTelemetry> {
    let mut findings = source_operations
        .iter()
        .filter_map(|entry| {
            let repeated_observations = entry
                .file_observation_count
                .saturating_sub(entry.unique_path_count as u64);
            (repeated_observations > 0).then(|| SearchBuildRepeatWorkFindingTelemetry {
                kind: "repeat_within_operation".to_string(),
                severity: "warning".to_string(),
                observations: entry.file_observation_count,
                repeated_observations,
                source: Some(entry.source.clone()),
                operation: Some(entry.operation.clone()),
                path: None,
                unique_path_count: Some(entry.unique_path_count),
                repeated_path_count: Some(entry.repeated_path_count),
                source_count: None,
                sources: vec![entry.source.clone()],
                operations: vec![entry.operation.clone()],
            })
        })
        .collect::<Vec<_>>();
    findings.extend(hot_paths.iter().map(|entry| {
        SearchBuildRepeatWorkFindingTelemetry {
            kind: if entry.source_count > 1 || entry.operations.len() > 1 {
                "cross_operation_hot_path".to_string()
            } else {
                "hot_path".to_string()
            },
            severity: if entry.source_count > 1 {
                "warning"
            } else {
                "info"
            }
            .to_string(),
            observations: entry.observations,
            repeated_observations: entry.observations.saturating_sub(1),
            source: None,
            operation: None,
            path: Some(entry.path.clone()),
            unique_path_count: None,
            repeated_path_count: None,
            source_count: Some(entry.source_count),
            sources: entry.sources.clone(),
            operations: entry.operations.clone(),
        }
    }));
    findings
}

fn top_findings(
    mut findings: Vec<SearchBuildRepeatWorkFindingTelemetry>,
) -> Vec<SearchBuildRepeatWorkFindingTelemetry> {
    findings.sort_by(|left, right| {
        severity_rank(right.severity.as_str())
            .cmp(&severity_rank(left.severity.as_str()))
            .then_with(|| right.repeated_observations.cmp(&left.repeated_observations))
            .then_with(|| right.observations.cmp(&left.observations))
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.source.cmp(&right.source))
            .then_with(|| left.operation.cmp(&right.operation))
    });
    findings.truncate(MAX_TELEMETRY_FINDINGS);
    findings
}

fn build_summary(
    total_file_observation_count: u64,
    total_unique_path_count: usize,
    source_operations: &[SearchBuildRepeatWorkSourceTelemetry],
    hot_paths: &[SearchBuildRepeatWorkHotPathTelemetry],
    findings: &[SearchBuildRepeatWorkFindingTelemetry],
) -> SearchBuildRepeatWorkSummaryTelemetry {
    SearchBuildRepeatWorkSummaryTelemetry {
        file_observations_total: total_file_observation_count,
        unique_paths_total: total_unique_path_count,
        repeated_file_observations: total_file_observation_count
            .saturating_sub(total_unique_path_count as u64),
        source_operations: source_operations.len(),
        hot_paths: hot_paths.len(),
        findings: findings.len(),
    }
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "warning" => 2,
        "info" => 1,
        _ => 0,
    }
}
