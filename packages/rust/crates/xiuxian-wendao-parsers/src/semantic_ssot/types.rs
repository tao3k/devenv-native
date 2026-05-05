//! Repo-native semantic SSOT types.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Semantic object kind admitted by the first SSOT slice.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticObjectKind {
    /// Durable architecture or runtime component.
    Component,
    /// Accepted or candidate architecture decision.
    Decision,
    /// Constraint that must remain true across changes.
    Invariant,
    /// Bounded execution or implementation task.
    Task,
}

impl SemanticObjectKind {
    /// Returns the canonical ID prefix for the object kind.
    #[must_use]
    pub fn id_prefix(&self) -> &'static str {
        match self {
            Self::Component => "component",
            Self::Decision => "decision",
            Self::Invariant => "invariant",
            Self::Task => "task",
        }
    }
}

/// Semantic lifecycle state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticStatus {
    /// Draft object not ready for default runtime scope.
    Draft,
    /// Candidate object that can be requested explicitly.
    Candidate,
    /// Active object included in default runtime scope.
    Active,
    /// Object superseded by a newer object.
    Superseded,
    /// Object retained for compatibility but no longer preferred.
    Deprecated,
    /// Retired object excluded from default runtime scope.
    Retired,
}

/// Source of a confidence declaration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticConfidenceSource {
    /// Accepted by repository governance.
    HumanSigned,
    /// Validated by a deterministic command or review gate.
    Verified,
    /// Proposed by an LLM and not authoritative by itself.
    LlmSuggested,
}

/// Confidence metadata attached to an object.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SemanticConfidence {
    /// Normalized confidence score from 0.0 through 1.0.
    pub score: f64,
    /// Source of the confidence declaration.
    pub source: SemanticConfidenceSource,
}

/// Owner metadata for one semantic object.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticOwner {
    /// Repository surface or package that owns the object.
    pub scope: String,
    /// Responsibility role for this object.
    pub role: String,
}

/// Provenance metadata for one semantic object.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticProvenance {
    /// Stable source document or artifact that justifies the object.
    pub source: String,
    /// Actor that recorded the object in the repo.
    pub recorded_by: String,
    /// Date or timestamp when the object was recorded.
    pub recorded_at: String,
}

/// Verification metadata for one semantic object.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticVerification {
    /// Required validation commands or checks for changes touching the object.
    pub required: Vec<String>,
    /// Stable evidence references for this object.
    #[serde(default)]
    pub evidence: Vec<String>,
}

/// Admitted relation kinds between semantic objects.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticRelationKind {
    /// Parent contains child.
    Contains,
    /// Source depends on target.
    DependsOn,
    /// Source constrains target.
    Constrains,
    /// Source implements target.
    Implements,
    /// Source governs target.
    Governs,
    /// Source affects target.
    Affects,
    /// Source validates target.
    Validates,
    /// Source supersedes target.
    Supersedes,
    /// Source projects to target.
    ProjectsTo,
    /// Source is consumed by target.
    ConsumedBy,
}

/// Relation declared inside one semantic object.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticRelation {
    /// Typed relation kind.
    pub kind: SemanticRelationKind,
    /// Target semantic object ID.
    pub target: String,
}

/// Operation declared for a semantic relation delta.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticRelationChangeAction {
    /// Add the relation.
    Add,
    /// Remove the relation.
    Remove,
    /// Update the relation semantics.
    Update,
}

/// Relation delta declared by a semantic change intent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticRelationChange {
    /// Source semantic object ID.
    pub source: String,
    /// Relation kind.
    pub kind: SemanticRelationKind,
    /// Target semantic object ID.
    pub target: String,
    /// Intended relation operation.
    pub action: SemanticRelationChangeAction,
}

/// Freshness state declared by a semantic projection artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticProjectionStaleness {
    /// Projection source revision matches the current source objects.
    Fresh,
    /// Projection is explicitly known to lag behind source objects.
    Stale,
}

/// Canonical semantic object loaded from Markdown frontmatter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SemanticObject {
    /// Stable semantic object ID.
    pub id: String,
    /// Object kind.
    pub kind: SemanticObjectKind,
    /// Human-readable object title.
    pub title: String,
    /// Lifecycle status.
    pub status: SemanticStatus,
    /// Confidence metadata.
    pub confidence: SemanticConfidence,
    /// Owner declarations.
    pub owners: Vec<SemanticOwner>,
    /// Provenance declaration.
    pub provenance: SemanticProvenance,
    /// Verification declaration.
    pub verification: SemanticVerification,
    /// Outgoing semantic relations.
    pub relations: Vec<SemanticRelation>,
    /// Markdown body after frontmatter.
    #[serde(default, skip_deserializing)]
    pub body: String,
    /// Path relative to the semantic root.
    #[serde(default, skip_deserializing)]
    pub source_path: PathBuf,
}

/// Semantic projection artifact loaded from Markdown frontmatter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticProjection {
    /// Projection artifact type. The first slice expects `semantic_projection`.
    #[serde(rename = "type")]
    pub projection_type: String,
    /// Projection name, such as `llm_compression`.
    pub projection: String,
    /// Source object IDs used by this projection.
    pub source_objects: Vec<String>,
    /// Deterministic source revision for the referenced source objects.
    pub source_revision: String,
    /// Stable projection revision identifier.
    pub projection_revision: String,
    /// Declared projection freshness relative to `source_revision`.
    pub staleness: SemanticProjectionStaleness,
    /// Lifecycle status for the projection artifact.
    pub status: SemanticStatus,
    /// Markdown body after frontmatter.
    #[serde(default, skip_deserializing)]
    pub body: String,
    /// Path relative to the semantic root.
    #[serde(default, skip_deserializing)]
    pub source_path: PathBuf,
}

/// Governance declaration for one semantic change.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticChangeIntent {
    /// Change-intent artifact type. The pilot expects `semantic_change_intent`.
    #[serde(rename = "type")]
    pub intent_type: String,
    /// Stable semantic change identifier.
    pub id: String,
    /// Human-readable title.
    pub title: String,
    /// Lifecycle status for this change declaration.
    pub status: SemanticStatus,
    /// Existing semantic objects touched by the change.
    pub touched_objects: Vec<String>,
    /// Intended relation deltas.
    #[serde(default)]
    pub changed_relations: Vec<SemanticRelationChange>,
    /// Existing invariant objects affected by the change.
    pub affected_invariants: Vec<String>,
    /// Required validation commands for closing the change.
    pub required_validations: Vec<String>,
    /// Projection names that must be refreshed or reviewed.
    pub projections_to_refresh: Vec<String>,
    /// Candidate semantic object IDs proposed by LLM or advisory processes.
    #[serde(default)]
    pub candidate_suggestions: Vec<String>,
    /// Markdown body after frontmatter.
    #[serde(default, skip_deserializing)]
    pub body: String,
    /// Path relative to the semantic root.
    #[serde(default, skip_deserializing)]
    pub source_path: PathBuf,
}

/// One validation issue for a semantic repository.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticValidationIssue {
    /// Path relative to the semantic root when the issue is path-specific.
    pub path: Option<PathBuf>,
    /// Human-readable issue message.
    pub message: String,
}

/// Validation report for a semantic repository.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticValidationReport {
    /// Collected validation issues.
    pub issues: Vec<SemanticValidationIssue>,
}

impl SemanticValidationReport {
    /// Returns true when no issues were collected.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.issues.is_empty()
    }

    pub(crate) fn push(&mut self, path: Option<PathBuf>, message: impl Into<String>) {
        self.issues.push(SemanticValidationIssue {
            path,
            message: message.into(),
        });
    }
}

/// Loaded semantic repository state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SemanticRepository {
    /// Root path passed to the loader.
    pub root: PathBuf,
    /// Loaded semantic objects.
    pub objects: Vec<SemanticObject>,
    /// Loaded projection artifacts.
    pub projections: Vec<SemanticProjection>,
    /// Loaded change-intent artifacts.
    pub change_intents: Vec<SemanticChangeIntent>,
    /// Validation report for objects, projections, and relations.
    pub report: SemanticValidationReport,
}

/// Request for a deterministic semantic scope bundle.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticScopeRequest {
    /// Optional task object ID that anchors the scope.
    pub task_id: Option<String>,
    /// Optional additional object IDs that anchor the scope.
    pub object_ids: Vec<String>,
}

/// Fully qualified relation edge in a scope bundle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticRelationEdge {
    /// Source semantic object ID.
    pub source: String,
    /// Relation kind.
    pub kind: SemanticRelationKind,
    /// Target semantic object ID.
    pub target: String,
}

/// Provenance summary for a scope bundle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticBundleProvenance {
    /// Source semantic object ID.
    pub object_id: String,
    /// Object source path.
    pub source_path: PathBuf,
    /// Source provenance reference from the object.
    pub source: String,
}

/// Deterministic semantic scope bundle returned to runtime consumers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SemanticScopeBundle {
    /// Optional task anchor ID.
    pub task_id: Option<String>,
    /// Object IDs requested explicitly by the caller.
    pub requested_object_ids: Vec<String>,
    /// Included semantic objects.
    pub objects: Vec<SemanticObject>,
    /// Included relation edges.
    pub relations: Vec<SemanticRelationEdge>,
    /// Included invariant object IDs.
    pub affected_invariants: Vec<String>,
    /// Deduplicated validation requirements from included objects.
    pub required_validations: Vec<String>,
    /// Projection revision that best represents this bundle.
    pub projection_revision: String,
    /// Source revision for the selected projection, when one exists.
    pub projection_source_revision: Option<String>,
    /// Freshness state for the selected projection, when one exists.
    pub projection_staleness: Option<SemanticProjectionStaleness>,
    /// Source provenance for included objects.
    pub provenance: Vec<SemanticBundleProvenance>,
    /// Requested IDs that could not be resolved.
    pub unresolved_ids: Vec<String>,
}
