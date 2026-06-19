//! `skill_runtime::manifest::types` owns Wendao skill runtime manifest types behavior.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Canonical URI prefix for skill runtime manifests.
pub const SKILL_RUNTIME_URI_PREFIX: &str = "wendao://skills";

/// Behavioral flags attached to tool annotations.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolBehaviorAnnotations {
    /// Operations that can be safely repeated without side effects.
    #[serde(default)]
    pub idempotent: bool,
    /// Operations that interact with external/open systems.
    #[serde(default)]
    pub open_world: bool,
}

/// Safety and behavior annotations for skill runtime aliases.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolAnnotations {
    /// Read-only operations that do not modify system state.
    #[serde(default)]
    pub read_only: bool,
    /// Operations that modify or delete data.
    #[serde(default)]
    pub destructive: bool,
    /// Behavioral flags flattened into the top-level annotation object.
    #[serde(flatten)]
    pub behavior: ToolBehaviorAnnotations,
}

impl ToolAnnotations {
    /// Return whether the tool is idempotent.
    #[must_use]
    pub const fn is_idempotent(&self) -> bool {
        self.behavior.idempotent
    }

    /// Set the idempotent flag.
    pub fn set_idempotent(&mut self, idempotent: bool) {
        self.behavior.idempotent = idempotent;
    }

    /// Return whether the tool can access open systems.
    #[must_use]
    pub const fn is_open_world(&self) -> bool {
        self.behavior.open_world
    }

    /// Set the open-world flag.
    pub fn set_open_world(&mut self, open_world: bool) {
        self.behavior.open_world = open_world;
    }
}

/// Supported skill workflow types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SkillWorkflowType {
    /// Execution driven by the `Qianji` orchestration engine.
    #[default]
    QianjiFlow,
    /// Direct dispatch to a native tool provider.
    NativeDispatch,
    /// Generic native tool execution.
    Native,
    /// Execution managed by an autonomous agent.
    Agentic,
}

impl SkillWorkflowType {
    /// Parse workflow type from raw string.
    #[must_use]
    pub fn from_raw(raw: Option<&str>) -> Self {
        let normalized = raw.unwrap_or("qianji_flow").trim().to_ascii_lowercase();
        match normalized.as_str() {
            "native_dispatch" | "native-dispatch" => Self::NativeDispatch,
            "native" => Self::Native,
            "agentic" => Self::Agentic,
            _ => Self::QianjiFlow,
        }
    }

    /// Return a stable string form for serialization.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::QianjiFlow => "qianji_flow",
            Self::NativeDispatch => "native_dispatch",
            Self::Native => "native",
            Self::Agentic => "agentic",
        }
    }
}

/// Free-form metadata attached to runtime skills.
pub type SkillMetadata = serde_json::Value;

/// Manifest parsed from runtime skill descriptors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillManifest {
    /// Unique identifier for the manifest.
    pub manifest_id: String,
    /// Human-readable tool name.
    pub tool_name: String,
    /// Detailed tool description.
    pub description: String,
    /// Type of execution workflow.
    pub workflow_type: SkillWorkflowType,
    /// Target runtime binding identifier.
    pub binding_id: String,
    /// Opaque metadata dictionary.
    pub metadata: SkillMetadata,
    /// Tool annotation overrides.
    pub annotations: ToolAnnotations,
    /// Absolute path to the source manifest file.
    pub source_path: std::path::PathBuf,
    /// Optional background context for rendering.
    #[serde(default)]
    pub context_background: Option<String>,
    /// Optional serialized flow definition.
    #[serde(default)]
    pub flow_definition: Option<String>,
}

/// Scan output for runtime skill manifests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SkillManifestScan {
    /// Paths to all discovered manifests.
    pub discovered_paths: Vec<std::path::PathBuf>,
    /// Successfully parsed and validated manifests.
    pub manifests: Vec<SkillManifest>,
    /// Collection of warnings or errors found during scanning.
    pub issues: Vec<String>,
}

/// Namespace boundary: this public name is scoped by its module owner.
/// Authority report for runtime skill manifest discovery.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SkillAuthorityReport {
    /// Manifests explicitly authorized by `SKILL.md` links.
    pub authorized_manifests: Vec<String>,
    /// `SKILL.md` links pointing to missing physical manifests.
    pub ghost_links: Vec<String>,
    /// Physical manifests not granted by any `SKILL.md`.
    pub unauthorized_manifests: Vec<String>,
}

/// Authority resolution output.
#[derive(Debug, Clone)]
pub struct SkillAuthorityOutcome {
    /// Detailed classification report.
    pub report: SkillAuthorityReport,
    /// Successfully loaded authorized manifest objects.
    pub authorized: Vec<SkillManifest>,
}

/// Error type for runtime skill manifest operations.
#[derive(Debug, Error)]
pub enum SkillManifestError {
    /// File read failure.
    #[error("failed to read skill manifest {path}: {source}")]
    Io {
        /// Source file path.
        path: String,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// TOML parsing or validation failure.
    #[error("failed to parse skill manifest {path}: {reason}")]
    Toml {
        /// Source file path.
        path: String,
        /// Human-readable reason for failure.
        reason: String,
    },
    /// Required manifest field is missing.
    #[error("skill manifest missing required field `{field}` at {path}")]
    MissingField {
        /// Source file path.
        path: String,
        /// Name of the missing field.
        field: String,
    },
    /// SKILL.md frontmatter is missing or violates the strict parser-owned schema.
    #[error("failed to parse SKILL.md frontmatter {path}: {reason}")]
    SkillFrontmatter {
        /// Source file path.
        path: String,
        /// Human-readable reason for failure.
        reason: String,
    },
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct ToolAnnotationsOverride {
    #[serde(default)]
    read_only: Option<bool>,
    #[serde(default)]
    destructive: Option<bool>,
    #[serde(default)]
    idempotent: Option<bool>,
    #[serde(default)]
    open_world: Option<bool>,
}

impl ToolAnnotationsOverride {
    pub(super) fn apply_defaults(self) -> ToolAnnotations {
        let mut annotations = ToolAnnotations {
            read_only: false,
            destructive: true,
            ..ToolAnnotations::default()
        };
        annotations.set_idempotent(false);
        annotations.set_open_world(true);
        if let Some(value) = self.read_only {
            annotations.read_only = value;
        }
        if let Some(value) = self.destructive {
            annotations.destructive = value;
        }
        if let Some(value) = self.idempotent {
            annotations.set_idempotent(value);
        }
        if let Some(value) = self.open_world {
            annotations.set_open_world(value);
        }
        annotations
    }
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct SkillManifestToml {
    #[serde(default)]
    pub(super) manifest_id: Option<String>,
    #[serde(default)]
    pub(super) id: Option<String>,
    #[serde(default)]
    pub(super) name: Option<String>,
    #[serde(default)]
    pub(super) tool_name: Option<String>,
    #[serde(default)]
    pub(super) binding_id: Option<String>,
    #[serde(default)]
    pub(super) description: Option<String>,
    #[serde(default)]
    pub(super) tool_contract: Option<serde_json::Value>,
    #[serde(default)]
    pub(super) contract: Option<serde_json::Value>,
    #[serde(default)]
    pub(super) workflow_type: Option<serde_json::Value>,
    #[serde(default)]
    pub(super) workflow: Option<serde_json::Value>,
    #[serde(default)]
    pub(super) context: Option<serde_json::Value>,
    #[serde(default)]
    pub(super) context_background: Option<serde_json::Value>,
    #[serde(default)]
    pub(super) background: Option<serde_json::Value>,
    #[serde(default)]
    pub(super) flow_definition: Option<serde_json::Value>,
    #[serde(default)]
    pub(super) flow: Option<serde_json::Value>,
    #[serde(default)]
    pub(super) annotations: Option<ToolAnnotationsOverride>,
    #[serde(default)]
    pub(super) tool_annotations: Option<ToolAnnotationsOverride>,
}
