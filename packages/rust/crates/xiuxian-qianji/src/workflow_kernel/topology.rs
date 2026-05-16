//! Front-end-neutral workflow topology contract.

use std::collections::{HashMap, VecDeque};

use super::{WorkflowEdgeKind, WorkflowStageStatus, WorkflowTrace};

/// Declares one stage that may be executed by a workflow run.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowStageBinding {
    /// Stable stage identifier.
    pub stage_id: String,
    /// Whether a checked finish must observe a successful execution for this
    /// stage.
    pub required: bool,
}

impl WorkflowStageBinding {
    /// Creates a required stage binding.
    #[must_use]
    pub fn required(stage_id: impl Into<String>) -> Self {
        Self {
            stage_id: stage_id.into(),
            required: true,
        }
    }

    /// Creates an optional stage binding.
    #[must_use]
    pub fn optional(stage_id: impl Into<String>) -> Self {
        Self {
            stage_id: stage_id.into(),
            required: false,
        }
    }
}

/// Declares a dependency edge between two workflow stages.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowTopologyEdge {
    /// Upstream stage id.
    pub from_stage_id: String,
    /// Downstream stage id.
    pub to_stage_id: String,
    /// Optional logical payload contract for this edge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edge_kind: Option<WorkflowEdgeKind>,
}

impl WorkflowTopologyEdge {
    /// Creates a topology edge without an explicit payload contract.
    #[must_use]
    pub fn new(from_stage_id: impl Into<String>, to_stage_id: impl Into<String>) -> Self {
        Self {
            from_stage_id: from_stage_id.into(),
            to_stage_id: to_stage_id.into(),
            edge_kind: None,
        }
    }

    /// Creates a topology edge with a typed or Arrow-backed payload contract.
    #[must_use]
    pub fn with_edge_kind(
        from_stage_id: impl Into<String>,
        to_stage_id: impl Into<String>,
        edge_kind: WorkflowEdgeKind,
    ) -> Self {
        Self {
            from_stage_id: from_stage_id.into(),
            to_stage_id: to_stage_id.into(),
            edge_kind: Some(edge_kind),
        }
    }
}

/// Front-end-neutral workflow topology.
///
/// BPMN and `petgraph` adapters can both compile to this model before binding
/// a typed [`WorkflowRun`](super::WorkflowRun).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowTopology {
    /// Stable workflow identifier.
    pub workflow_id: String,
    /// Declared workflow stages.
    pub stages: Vec<WorkflowStageBinding>,
    /// Declared dependency edges.
    #[serde(default)]
    pub edges: Vec<WorkflowTopologyEdge>,
}

impl WorkflowTopology {
    /// Creates a workflow topology.
    #[must_use]
    pub fn new(
        workflow_id: impl Into<String>,
        stages: Vec<WorkflowStageBinding>,
        edges: Vec<WorkflowTopologyEdge>,
    ) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            stages,
            edges,
        }
    }

    /// Creates a required linear topology from an ordered stage list.
    ///
    /// # Errors
    ///
    /// Returns an error when the topology is invalid.
    pub fn linear<I, S>(
        workflow_id: impl Into<String>,
        stage_ids: I,
    ) -> Result<Self, WorkflowTopologyError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let stages = stage_ids
            .into_iter()
            .map(|stage_id| WorkflowStageBinding::required(stage_id.into()))
            .collect::<Vec<_>>();
        let edges = stages
            .windows(2)
            .map(|window| {
                WorkflowTopologyEdge::new(window[0].stage_id.clone(), window[1].stage_id.clone())
            })
            .collect::<Vec<_>>();
        let topology = Self::new(workflow_id, stages, edges);
        topology.validate()?;
        Ok(topology)
    }

    /// Validates this topology.
    ///
    /// # Errors
    ///
    /// Returns an error for empty ids, duplicate stages, missing edge
    /// endpoints, or dependency cycles.
    pub fn validate(&self) -> Result<(), WorkflowTopologyError> {
        self.topological_stage_ids().map(|_| ())
    }

    /// Returns whether this topology declares the supplied stage id.
    #[must_use]
    pub fn contains_stage(&self, stage_id: &str) -> bool {
        self.stages.iter().any(|stage| stage.stage_id == stage_id)
    }

    /// Returns required stage ids.
    pub fn required_stage_ids(&self) -> impl Iterator<Item = &str> {
        self.stages
            .iter()
            .filter(|stage| stage.required)
            .map(|stage| stage.stage_id.as_str())
    }

    /// Returns a topological stage order.
    ///
    /// # Errors
    ///
    /// Returns an error when the topology is invalid.
    pub fn topological_stage_ids(&self) -> Result<Vec<String>, WorkflowTopologyError> {
        if self.workflow_id.trim().is_empty() {
            return Err(WorkflowTopologyError::EmptyWorkflowId);
        }
        if self.stages.is_empty() {
            return Err(WorkflowTopologyError::EmptyStages {
                workflow_id: self.workflow_id.clone(),
            });
        }

        let mut stage_index = HashMap::with_capacity(self.stages.len());
        for (index, stage) in self.stages.iter().enumerate() {
            if stage.stage_id.trim().is_empty() {
                return Err(WorkflowTopologyError::EmptyStageId {
                    workflow_id: self.workflow_id.clone(),
                });
            }
            if stage_index.insert(stage.stage_id.as_str(), index).is_some() {
                return Err(WorkflowTopologyError::DuplicateStage {
                    workflow_id: self.workflow_id.clone(),
                    stage_id: stage.stage_id.clone(),
                });
            }
        }

        let mut adjacency = vec![Vec::new(); self.stages.len()];
        let mut indegree = vec![0_usize; self.stages.len()];
        for edge in &self.edges {
            let Some(from_index) = stage_index.get(edge.from_stage_id.as_str()).copied() else {
                return Err(WorkflowTopologyError::MissingEdgeStage {
                    workflow_id: self.workflow_id.clone(),
                    from_stage_id: edge.from_stage_id.clone(),
                    to_stage_id: edge.to_stage_id.clone(),
                    missing_stage_id: edge.from_stage_id.clone(),
                });
            };
            let Some(to_index) = stage_index.get(edge.to_stage_id.as_str()).copied() else {
                return Err(WorkflowTopologyError::MissingEdgeStage {
                    workflow_id: self.workflow_id.clone(),
                    from_stage_id: edge.from_stage_id.clone(),
                    to_stage_id: edge.to_stage_id.clone(),
                    missing_stage_id: edge.to_stage_id.clone(),
                });
            };
            adjacency[from_index].push(to_index);
            indegree[to_index] += 1;
        }

        let mut ready = indegree
            .iter()
            .enumerate()
            .filter_map(|(index, degree)| (*degree == 0).then_some(index))
            .collect::<VecDeque<_>>();
        let mut ordered = Vec::with_capacity(self.stages.len());

        while let Some(index) = ready.pop_front() {
            ordered.push(self.stages[index].stage_id.clone());
            for next_index in &adjacency[index] {
                indegree[*next_index] -= 1;
                if indegree[*next_index] == 0 {
                    ready.push_back(*next_index);
                }
            }
        }

        if ordered.len() != self.stages.len() {
            return Err(WorkflowTopologyError::Cycle {
                workflow_id: self.workflow_id.clone(),
            });
        }

        Ok(ordered)
    }

    /// Validates an execution trace against this topology.
    ///
    /// # Errors
    ///
    /// Returns an error when the trace contains undeclared stages, duplicate
    /// successful stages, missing required stages, or dependency order
    /// violations.
    pub fn validate_trace(&self, trace: &WorkflowTrace) -> Result<(), WorkflowCompletionError> {
        let mut successful_indices = HashMap::new();
        for (index, stage_trace) in trace.stages.iter().enumerate() {
            if !self.contains_stage(stage_trace.stage_id.as_str()) {
                return Err(WorkflowCompletionError::UndeclaredStage {
                    workflow_id: self.workflow_id.clone(),
                    stage_id: stage_trace.stage_id.clone(),
                    trace: trace.clone(),
                });
            }
            if stage_trace.status == WorkflowStageStatus::Succeeded
                && successful_indices
                    .insert(stage_trace.stage_id.as_str(), index)
                    .is_some()
            {
                return Err(WorkflowCompletionError::DuplicateSuccessfulStage {
                    workflow_id: self.workflow_id.clone(),
                    stage_id: stage_trace.stage_id.clone(),
                    trace: trace.clone(),
                });
            }
        }

        let missing_stage_ids = self
            .required_stage_ids()
            .filter(|stage_id| !successful_indices.contains_key(stage_id))
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if !missing_stage_ids.is_empty() {
            return Err(WorkflowCompletionError::MissingRequiredStages {
                workflow_id: self.workflow_id.clone(),
                missing_stage_ids,
                trace: trace.clone(),
            });
        }

        for edge in &self.edges {
            let Some(from_index) = successful_indices.get(edge.from_stage_id.as_str()) else {
                continue;
            };
            let Some(to_index) = successful_indices.get(edge.to_stage_id.as_str()) else {
                continue;
            };
            if from_index >= to_index {
                return Err(WorkflowCompletionError::EdgeOrderViolation {
                    workflow_id: self.workflow_id.clone(),
                    from_stage_id: edge.from_stage_id.clone(),
                    to_stage_id: edge.to_stage_id.clone(),
                    trace: trace.clone(),
                });
            }
        }

        Ok(())
    }
}

/// Error returned when a topology contract is invalid.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum WorkflowTopologyError {
    /// The workflow id is empty.
    #[error("workflow topology id cannot be empty")]
    EmptyWorkflowId,
    /// The topology does not declare any stages.
    #[error("workflow topology `{workflow_id}` must declare at least one stage")]
    EmptyStages {
        /// Stable workflow identifier.
        workflow_id: String,
    },
    /// A stage id is empty.
    #[error("workflow topology `{workflow_id}` contains an empty stage id")]
    EmptyStageId {
        /// Stable workflow identifier.
        workflow_id: String,
    },
    /// A stage id appears more than once.
    #[error("workflow topology `{workflow_id}` declares duplicate stage `{stage_id}`")]
    DuplicateStage {
        /// Stable workflow identifier.
        workflow_id: String,
        /// Duplicate stage id.
        stage_id: String,
    },
    /// A dependency edge references a stage that is not declared.
    #[error(
        "workflow topology `{workflow_id}` edge `{from_stage_id}` -> `{to_stage_id}` references missing stage `{missing_stage_id}`"
    )]
    MissingEdgeStage {
        /// Stable workflow identifier.
        workflow_id: String,
        /// Upstream stage id from the invalid edge.
        from_stage_id: String,
        /// Downstream stage id from the invalid edge.
        to_stage_id: String,
        /// Missing stage id.
        missing_stage_id: String,
    },
    /// The dependency graph contains a cycle.
    #[error("workflow topology `{workflow_id}` contains a dependency cycle")]
    Cycle {
        /// Stable workflow identifier.
        workflow_id: String,
    },
}

/// Error returned when a completed trace violates its bound topology.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum WorkflowCompletionError {
    /// A trace contains a stage not declared by topology.
    #[error("workflow `{workflow_id}` executed undeclared stage `{stage_id}`")]
    UndeclaredStage {
        /// Stable workflow identifier.
        workflow_id: String,
        /// Undeclared stage id.
        stage_id: String,
        /// Trace captured through validation.
        trace: WorkflowTrace,
    },
    /// A trace contains a successful stage more than once.
    #[error("workflow `{workflow_id}` executed stage `{stage_id}` successfully more than once")]
    DuplicateSuccessfulStage {
        /// Stable workflow identifier.
        workflow_id: String,
        /// Duplicate stage id.
        stage_id: String,
        /// Trace captured through validation.
        trace: WorkflowTrace,
    },
    /// Required stages did not complete successfully.
    #[error("workflow `{workflow_id}` missing required stage(s): {missing_stage_ids:?}")]
    MissingRequiredStages {
        /// Stable workflow identifier.
        workflow_id: String,
        /// Missing required stage ids.
        missing_stage_ids: Vec<String>,
        /// Trace captured through validation.
        trace: WorkflowTrace,
    },
    /// A dependency edge was completed out of order.
    #[error("workflow `{workflow_id}` violated edge order `{from_stage_id}` -> `{to_stage_id}`")]
    EdgeOrderViolation {
        /// Stable workflow identifier.
        workflow_id: String,
        /// Upstream stage id.
        from_stage_id: String,
        /// Downstream stage id.
        to_stage_id: String,
        /// Trace captured through validation.
        trace: WorkflowTrace,
    },
}
