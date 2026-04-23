use std::fs;
use std::path::Path;

use crate::error::QianjiError;
use crate::flowhub::MermaidFlowchart;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkdirRuntimeState {
    pub(crate) current_node: WorkdirCurrentNodeState,
    pub(crate) allowed_next: WorkdirAllowedNextState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkdirCurrentNodeState {
    pub(crate) raw_ref: Option<String>,
    pub(crate) resolved: Option<WorkdirRuntimeNode>,
    pub(crate) issue: Option<WorkdirCurrentNodeIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkdirAllowedNextState {
    pub(crate) raw_refs: Option<Vec<String>>,
    pub(crate) resolved_labels: Vec<String>,
    pub(crate) expected_labels: Vec<String>,
    pub(crate) issue: Option<WorkdirAllowedNextIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkdirRuntimeNode {
    pub(crate) id: String,
    pub(crate) label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WorkdirCurrentNodeIssue {
    MissingField,
    UnknownNode(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WorkdirAllowedNextIssue {
    InvalidJson(String),
    UnknownNode(String),
}

pub(crate) fn load_workdir_runtime_state(
    workdir: &Path,
    flowchart: &MermaidFlowchart,
) -> Result<WorkdirRuntimeState, QianjiError> {
    let current_node_path = workdir.join("state/current_node.toml");
    let current_node = if current_node_path.is_file() {
        match parse_current_node_ref(&current_node_path)? {
            Some(raw_ref) => WorkdirCurrentNodeState {
                resolved: resolve_runtime_node(flowchart, raw_ref.as_str()),
                issue: None,
                raw_ref: Some(raw_ref),
            },
            None => WorkdirCurrentNodeState {
                raw_ref: None,
                resolved: None,
                issue: Some(WorkdirCurrentNodeIssue::MissingField),
            },
        }
    } else {
        WorkdirCurrentNodeState {
            raw_ref: None,
            resolved: None,
            issue: None,
        }
    };

    let current_node = if current_node.issue.is_none() && current_node.resolved.is_none() {
        if let Some(raw_ref) = current_node.raw_ref.as_ref() {
            WorkdirCurrentNodeState {
                issue: Some(WorkdirCurrentNodeIssue::UnknownNode(raw_ref.clone())),
                ..current_node
            }
        } else {
            current_node
        }
    } else {
        current_node
    };

    let allowed_next_path = workdir.join("state/allowed_next.json");
    let allowed_next = if allowed_next_path.is_file() {
        let allowed_next_json = fs::read_to_string(&allowed_next_path).map_err(|error| {
            QianjiError::Topology(format!(
                "Failed to read localized allowed-next state `{}`: {error}",
                allowed_next_path.display()
            ))
        })?;
        match serde_json::from_str::<Vec<String>>(&allowed_next_json) {
            Ok(raw_refs) => {
                let mut resolved_labels = Vec::new();
                let mut issue = None;
                for next_ref in &raw_refs {
                    if let Some(node) = resolve_runtime_node(flowchart, next_ref.as_str()) {
                        push_unique_string(&mut resolved_labels, node.label.as_str());
                    } else {
                        issue = Some(WorkdirAllowedNextIssue::UnknownNode(next_ref.clone()));
                        break;
                    }
                }
                let mut expected_labels = current_node
                    .resolved
                    .as_ref()
                    .map_or_else(Vec::new, |current| {
                        expected_next_labels(flowchart, current.id.as_str())
                    });
                expected_labels.sort();
                resolved_labels.sort();
                WorkdirAllowedNextState {
                    raw_refs: Some(raw_refs),
                    resolved_labels,
                    expected_labels,
                    issue,
                }
            }
            Err(error) => WorkdirAllowedNextState {
                raw_refs: None,
                resolved_labels: Vec::new(),
                expected_labels: Vec::new(),
                issue: Some(WorkdirAllowedNextIssue::InvalidJson(error.to_string())),
            },
        }
    } else {
        WorkdirAllowedNextState {
            raw_refs: None,
            resolved_labels: Vec::new(),
            expected_labels: Vec::new(),
            issue: None,
        }
    };

    Ok(WorkdirRuntimeState {
        current_node,
        allowed_next,
    })
}

pub(crate) fn resolve_runtime_node(
    flowchart: &MermaidFlowchart,
    node_ref: &str,
) -> Option<WorkdirRuntimeNode> {
    let exact_matches = flowchart
        .nodes
        .iter()
        .filter(|node| node.id == node_ref || node.label == node_ref)
        .map(|node| WorkdirRuntimeNode {
            id: node.id.clone(),
            label: node.label.clone(),
        })
        .collect::<Vec<_>>();
    if exact_matches.len() == 1 {
        return exact_matches.into_iter().next();
    }

    let normalized_ref = normalize_node_ref(node_ref);
    let normalized_matches = flowchart
        .nodes
        .iter()
        .filter(|node| normalize_node_ref(node.label.as_str()) == normalized_ref)
        .map(|node| WorkdirRuntimeNode {
            id: node.id.clone(),
            label: node.label.clone(),
        })
        .collect::<Vec<_>>();
    if normalized_matches.len() == 1 {
        return normalized_matches.into_iter().next();
    }

    None
}

fn parse_current_node_ref(current_node_path: &Path) -> Result<Option<String>, QianjiError> {
    let current_node_toml = fs::read_to_string(current_node_path).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to read localized current-node state `{}`: {error}",
            current_node_path.display()
        ))
    })?;
    let value = toml::from_str::<toml::Value>(&current_node_toml).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to parse localized current-node state `{}`: {error}",
            current_node_path.display()
        ))
    })?;

    Ok(value
        .get("current_node")
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string))
}

pub(crate) fn expected_next_labels(
    flowchart: &MermaidFlowchart,
    current_node_id: &str,
) -> Vec<String> {
    flowchart
        .edges
        .iter()
        .filter(|edge| edge.from == current_node_id)
        .filter_map(|edge| {
            flowchart
                .nodes
                .iter()
                .find(|node| node.id == edge.to)
                .map(|node| node.label.clone())
        })
        .collect()
}

fn normalize_node_ref(node_ref: &str) -> String {
    node_ref
        .chars()
        .flat_map(char::to_lowercase)
        .filter(char::is_ascii_alphanumeric)
        .collect()
}

fn push_unique_string(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|entry| entry == value) {
        values.push(value.to_string());
    }
}
