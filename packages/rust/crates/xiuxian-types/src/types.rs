//! xiuxian-types - Common type definitions for Xiuxian `DevEnv`
//!
//! This crate provides shared data structures used across Xiuxian and Omni crates.
//! All types are designed to be serialization-compatible with Python (via `PyO3`).
//!
//! # Schema Singularity
//! Types derive `schemars::JsonSchema` to enable automatic JSON Schema generation.
//! This establishes Rust as the Single Source of Truth (SSOT) for type definitions,
//! allowing Python and LLM consumers to dynamically retrieve authoritative schemas.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type with project-specific error.
pub type OmniResult<T> = Result<T, OmniError>;

/// Unified error type for all Omni operations
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum OmniError {
    /// Git-related operation failures
    #[error("Git error: {0}")]
    Git(String),

    /// File system access failures
    #[error("File system error: {0}")]
    Filesystem(String),

    /// Configuration loading/parsing failures
    #[error("Configuration error: {0}")]
    Config(String),

    /// Unclassified failures
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Agent skill definition
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Skill {
    /// Skill name identifier
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Skill category
    pub category: String,
}

/// Skill definition with generic metadata container.
/// This enables schema-driven metadata evolution without recompiling Rust.
///
/// All schema-defined fields (version, permissions, `require_refs`, etc.)
/// are stored in the flexible `metadata` JSON object.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(from = "SkillDefinitionHelper", into = "SkillDefinitionHelper")]
pub struct SkillDefinition {
    /// Unique identifier for the skill (e.g., "git", "writer")
    pub name: String,
    /// Semantic description used for vector embedding generation
    pub description: String,
    /// Generic metadata container for schema-defined fields
    pub metadata: serde_json::Value,
    /// Routing keywords for semantic search
    #[serde(default)]
    pub routing_keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct SkillDefinitionHelper {
    name: String,
    description: String,
    #[serde(default = "default_metadata")]
    metadata: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    routing_keywords: Option<Vec<String>>,
}

fn default_metadata() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

fn normalize_string_list(values: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !normalized.iter().any(|existing| existing == trimmed) {
            normalized.push(trimmed.to_string());
        }
    }
    normalized
}

fn extract_string_list(value: &serde_json::Value) -> Vec<String> {
    let values = match value {
        serde_json::Value::String(value) => vec![value.clone()],
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    };
    normalize_string_list(values)
}

fn camel_case_key(key: &str) -> Option<String> {
    if !key.contains('_') && !key.contains('-') {
        return None;
    }
    let mut iter = key.split(['_', '-']).filter(|part| !part.is_empty());
    let first = iter.next()?;
    let mut out = first.to_ascii_lowercase();
    for part in iter {
        let mut chars = part.chars();
        if let Some(first_char) = chars.next() {
            out.extend(first_char.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    Some(out)
}

fn pascal_case_key(key: &str) -> Option<String> {
    if key.is_empty() {
        return None;
    }
    let mut out = String::new();
    if key.contains('_') || key.contains('-') {
        for part in key.split(['_', '-']).filter(|part| !part.is_empty()) {
            let mut chars = part.chars();
            if let Some(first_char) = chars.next() {
                out.extend(first_char.to_uppercase());
                out.push_str(chars.as_str());
            }
        }
        return Some(out);
    }
    let mut chars = key.chars();
    let first_char = chars.next()?;
    out.extend(first_char.to_uppercase());
    out.push_str(chars.as_str());
    Some(out)
}

fn metadata_key_variants(key: &str) -> Vec<String> {
    let mut variants = Vec::new();
    variants.push(key.to_string());
    if let Some(camel) = camel_case_key(key)
        && camel != key
    {
        variants.push(camel);
    }
    if let Some(pascal) = pascal_case_key(key)
        && !variants.iter().any(|existing| existing == &pascal)
    {
        variants.push(pascal);
    }
    variants
}

fn metadata_get<'a>(metadata: &'a serde_json::Value, key: &str) -> Option<&'a serde_json::Value> {
    let object = metadata.as_object()?;
    for variant in metadata_key_variants(key) {
        if let Some(value) = object.get(&variant) {
            return Some(value);
        }
    }
    None
}

fn metadata_get_string(metadata: &serde_json::Value, key: &str) -> Option<String> {
    metadata_get(metadata, key).and_then(|value| value.as_str().map(str::to_string))
}

fn metadata_get_string_list(metadata: &serde_json::Value, key: &str) -> Vec<String> {
    metadata_get(metadata, key)
        .map(extract_string_list)
        .unwrap_or_default()
}

fn metadata_get_string_list_variants(metadata: &serde_json::Value, key: &str) -> Vec<String> {
    let Some(object) = metadata.as_object() else {
        return Vec::new();
    };
    let mut values = Vec::new();
    for variant in metadata_key_variants(key) {
        if let Some(value) = object.get(&variant) {
            values.extend(extract_string_list(value));
        }
    }
    normalize_string_list(values)
}

#[allow(dead_code)]
fn metadata_contains_key(metadata: &serde_json::Value, key: &str) -> bool {
    let Some(object) = metadata.as_object() else {
        return false;
    };
    metadata_key_variants(key)
        .iter()
        .any(|variant| object.contains_key(variant))
}

fn ensure_metadata_object(
    metadata: &mut serde_json::Value,
) -> &mut serde_json::Map<String, serde_json::Value> {
    if !metadata.is_object() {
        *metadata = serde_json::Value::Object(serde_json::Map::new());
    }
    if metadata.as_object_mut().is_none() {
        *metadata = serde_json::Value::Object(serde_json::Map::new());
    }
    match metadata.as_object_mut() {
        Some(object) => object,
        None => unreachable!("metadata should be an object after initialization"),
    }
}

fn extract_routing_keywords(
    metadata: &serde_json::Value,
    explicit: Option<Vec<String>>,
) -> Vec<String> {
    let mut keywords = metadata_get_string_list_variants(metadata, "routing_keywords");
    if let Some(extra) = explicit {
        keywords.extend(extra);
    }
    normalize_string_list(keywords)
}

fn ensure_metadata_routing_keywords(metadata: &mut serde_json::Value, routing_keywords: &[String]) {
    let normalized = normalize_string_list(routing_keywords.to_vec());
    if normalized.is_empty() {
        return;
    }
    let mut merged = metadata_get_string_list_variants(metadata, "routing_keywords");
    merged.extend(normalized);
    let merged = normalize_string_list(merged);
    if merged.is_empty() {
        return;
    }
    let object = ensure_metadata_object(metadata);
    for variant in metadata_key_variants("routing_keywords") {
        object.remove(&variant);
    }
    object.insert(
        "routing_keywords".to_string(),
        serde_json::Value::Array(merged.into_iter().map(serde_json::Value::String).collect()),
    );
}

impl From<SkillDefinitionHelper> for SkillDefinition {
    fn from(helper: SkillDefinitionHelper) -> Self {
        let metadata = helper.metadata.clone();
        let routing_keywords = extract_routing_keywords(&metadata, helper.routing_keywords);

        Self {
            name: helper.name,
            description: helper.description,
            metadata,
            routing_keywords,
        }
    }
}

impl From<SkillDefinition> for SkillDefinitionHelper {
    fn from(def: SkillDefinition) -> Self {
        let mut metadata = def.metadata;
        ensure_metadata_routing_keywords(&mut metadata, &def.routing_keywords);
        Self {
            name: def.name,
            description: def.description,
            metadata,
            routing_keywords: None,
        }
    }
}

impl SkillDefinition {
    /// Create a new skill definition.
    #[must_use]
    pub fn new(name: String, description: String, metadata: serde_json::Value) -> Self {
        let routing_keywords = extract_routing_keywords(&metadata, None);

        Self {
            name,
            description,
            metadata,
            routing_keywords,
        }
    }

    /// Get `require_refs` from metadata safely.
    #[must_use]
    pub fn get_require_refs(&self) -> Vec<String> {
        metadata_get_string_list(&self.metadata, "require_refs")
    }

    /// Get a specific metadata field as string.
    /// Tries `snake_case`, camelCase, and `PascalCase` variations.
    #[must_use]
    pub fn get_meta_string(&self, key: &str) -> Option<String> {
        metadata_get_string(&self.metadata, key)
    }

    /// Get skill version from metadata.
    #[must_use]
    pub fn get_version(&self) -> String {
        self.get_meta_string("version").unwrap_or_default()
    }
}

/// Task brief from orchestrator
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskBrief {
    /// Task description
    pub task: String,
    /// Mission objectives
    pub mission_brief: String,
    /// Constraints to follow
    pub constraints: Vec<String>,
    /// Files relevant to this task
    pub relevant_files: Vec<String>,
}

/// Agent execution result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentResult {
    /// Whether the task succeeded
    pub success: bool,
    /// Result content
    pub content: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Human-readable message
    pub message: String,
}

/// Context for agent execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentContext {
    /// System prompt for the agent
    pub system_prompt: String,
    /// Available tools/skills
    pub tools: Vec<Skill>,
    /// Mission brief
    pub mission_brief: String,
    /// Constraints
    pub constraints: Vec<String>,
    /// Relevant files
    pub relevant_files: Vec<String>,
}

/// Canonical knowledge category used across scanners, storage, and query APIs.
///
/// This enum accepts both singular and plural spellings during deserialization:
/// for example, `pattern` and `patterns` both map to `Pattern`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeCategory {
    /// Architecture design and structural decisions.
    Architecture,
    /// Debugging guides and diagnosis notes.
    Debugging,
    /// Error diagnosis and fixes.
    #[serde(alias = "errors")]
    Error,
    /// General notes.
    #[serde(alias = "notes")]
    #[default]
    Note,
    /// Reusable patterns and practices.
    #[serde(alias = "patterns")]
    Pattern,
    /// Reference material.
    #[serde(alias = "references")]
    Reference,
    /// Techniques and how-to methods.
    #[serde(alias = "techniques")]
    Technique,
    /// Workflow and process guidance.
    #[serde(alias = "workflows")]
    Workflow,
    /// Problem-solution writeups.
    #[serde(alias = "solutions")]
    Solution,
    /// Uncategorized or unknown value.
    #[serde(other)]
    Unknown,
}

impl KnowledgeCategory {
    /// Canonical singular label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Architecture => "architecture",
            Self::Debugging => "debugging",
            Self::Error => "error",
            Self::Note => "note",
            Self::Pattern => "pattern",
            Self::Reference => "reference",
            Self::Technique => "technique",
            Self::Workflow => "workflow",
            Self::Solution => "solution",
            Self::Unknown => "unknown",
        }
    }

    /// Canonical plural/storage label.
    #[must_use]
    pub const fn as_plural_str(self) -> &'static str {
        match self {
            Self::Architecture => "architecture",
            Self::Debugging => "debugging",
            Self::Error => "errors",
            Self::Note => "notes",
            Self::Pattern => "patterns",
            Self::Reference => "references",
            Self::Technique => "techniques",
            Self::Workflow => "workflows",
            Self::Solution => "solutions",
            Self::Unknown => "unknown",
        }
    }
}

impl std::str::FromStr for KnowledgeCategory {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let normalized = input.trim().to_ascii_lowercase();
        Ok(match normalized.as_str() {
            "architecture" | "arch" => Self::Architecture,
            "debugging" | "debug" => Self::Debugging,
            "error" | "errors" | "err" => Self::Error,
            "note" | "notes" => Self::Note,
            "pattern" | "patterns" => Self::Pattern,
            "reference" | "references" | "ref" => Self::Reference,
            "technique" | "techniques" => Self::Technique,
            "workflow" | "workflows" => Self::Workflow,
            "solution" | "solutions" => Self::Solution,
            _ => Self::Unknown,
        })
    }
}

impl std::fmt::Display for KnowledgeCategory {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.as_str())
    }
}

/// 3-in-1 gate verdict for memory lifecycle transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MemoryGateVerdict {
    /// Keep the episode in short-term memory.
    Retain,
    /// Remove the episode from memory storage.
    Obsolete,
    /// Promote the episode out of episodic memory. Consult `promotion_target`
    /// for the destination layer.
    Promote,
}

impl MemoryGateVerdict {
    /// String form used in contracts/logs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Retain => "retain",
            Self::Obsolete => "obsolete",
            Self::Promote => "promote",
        }
    }
}

/// Explicit destination for memory promotion decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MemoryPromotionTarget {
    /// Promote into the short-term working-knowledge layer.
    WorkingKnowledge,
}

impl MemoryPromotionTarget {
    /// String form used in contracts/logs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WorkingKnowledge => "working_knowledge",
        }
    }
}

/// Evidence-based gate decision payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MemoryGateDecision {
    /// Final verdict.
    pub verdict: MemoryGateVerdict,
    /// Confidence score in [0, 1].
    pub confidence: f32,
    /// `ReAct` evidence references.
    pub react_evidence_refs: Vec<String>,
    /// Graph evidence references.
    pub graph_evidence_refs: Vec<String>,
    /// Omega factors and notes.
    pub omega_factors: Vec<String>,
    /// Audit reason.
    pub reason: String,
    /// Next action command.
    pub next_action: String,
    /// Explicit promotion target for `promote` verdicts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub promotion_target: Option<MemoryPromotionTarget>,
}

/// Vector search result (Arrow-native fields preferred; metadata kept for filter/extra keys).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct VectorSearchResult {
    /// Result identifier (e.g. tool name)
    pub id: String,
    /// Result content
    pub content: String,
    /// Tool name (Arrow-native; avoid parsing metadata)
    #[serde(default)]
    pub tool_name: String,
    /// File path (Arrow-native)
    #[serde(default)]
    pub file_path: String,
    /// Routing keywords as string (space/whitespace-separated; Arrow-native)
    #[serde(default)]
    pub routing_keywords: String,
    /// Intents as string (e.g. " | "-separated; Arrow-native)
    #[serde(default)]
    pub intents: String,
    /// Additional metadata (for filter and extra keys; may be built from native columns)
    pub metadata: serde_json::Value,
    /// Distance from query vector
    pub distance: f64,
}

/// Environment snapshot for the sensory system.
/// This is the Rosetta Stone for Rust-Python communication.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnvironmentSnapshot {
    /// Current Git branch name
    pub git_branch: String,
    /// Number of modified (unstaged) files
    pub git_modified: usize,
    /// Number of staged files
    pub git_staged: usize,
    /// Number of lines in active context (SCRATCHPAD.md)
    pub active_context_lines: usize,
    /// List of modified file paths
    pub dirty_files: Vec<String>,
    /// Unix timestamp of snapshot creation
    pub timestamp: f64,
}

impl Default for EnvironmentSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentSnapshot {
    /// Create a new empty environment snapshot.
    #[must_use]
    pub fn new() -> Self {
        Self {
            git_branch: "unknown".to_string(),
            git_modified: 0,
            git_staged: 0,
            active_context_lines: 0,
            dirty_files: vec![],
            timestamp: 0.0,
        }
    }

    /// Render as human-readable prompt string for Agent consumption.
    #[must_use]
    pub fn to_prompt_string(&self) -> String {
        let dirty_desc = if self.dirty_files.is_empty() {
            "Clean".to_string()
        } else {
            let count = self.dirty_files.len();
            let preview = self
                .dirty_files
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            if count > 3 {
                format!("{count} files ({preview}, ...)")
            } else {
                format!("{count} files ({preview})")
            }
        };

        format!(
            "[LIVE ENVIRONMENT STATE]\n\
            - Git: Branch: {} | Modified: {} | Staged: {} | Status: {}\n\
            - Active Context: {} lines in SCRATCHPAD.md",
            self.git_branch,
            self.git_modified,
            self.git_staged,
            dirty_desc,
            self.active_context_lines
        )
    }
}

// =============================================================================
// Schema Registry: Dynamic JSON Schema Generation for Python/LLM Consumption
// =============================================================================

/// Schema generation error
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    /// The requested type name is not registered.
    #[error("Unknown type: {0}")]
    UnknownType(String),
    /// JSON Schema serialization failed.
    #[error("Schema serialization failed: {0}")]
    Serialization(String),
}

/// Get JSON Schema for a registered type.
/// This enables Python to dynamically retrieve authoritative schemas from Rust.
///
/// # Errors
/// Returns `SchemaError::UnknownType` if the type name is not registered.
pub fn get_schema_json(type_name: &str) -> Result<String, SchemaError> {
    let schema = match type_name {
        // Core types
        "Skill" => schemars::schema_for!(Skill),
        "SkillDefinition" | "OmniTool" => schemars::schema_for!(SkillDefinition),
        "TaskBrief" => schemars::schema_for!(TaskBrief),
        "AgentResult" => schemars::schema_for!(AgentResult),
        "AgentContext" => schemars::schema_for!(AgentContext),
        "KnowledgeCategory" => schemars::schema_for!(KnowledgeCategory),
        "MemoryGateVerdict" => schemars::schema_for!(MemoryGateVerdict),
        "MemoryGateDecision" => schemars::schema_for!(MemoryGateDecision),
        "VectorSearchResult" => schemars::schema_for!(VectorSearchResult),
        "EnvironmentSnapshot" => schemars::schema_for!(EnvironmentSnapshot),
        _ => return Err(SchemaError::UnknownType(type_name.to_string())),
    };
    serde_json::to_string_pretty(&schema).map_err(|e| SchemaError::Serialization(e.to_string()))
}

/// Get list of all registered type names.
#[must_use]
pub fn get_registered_types() -> Vec<&'static str> {
    vec![
        "Skill",
        "SkillDefinition",
        "TaskBrief",
        "AgentResult",
        "AgentContext",
        "KnowledgeCategory",
        "MemoryGateVerdict",
        "MemoryGateDecision",
        "VectorSearchResult",
        "EnvironmentSnapshot",
        "OmniTool", // Alias for SkillDefinition
    ]
}
